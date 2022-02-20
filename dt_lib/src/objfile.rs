/// Functions for parsing OMF (Object Module Format). An OMF file 
/// (commonly known as an obj file) contains multiple record, each
/// with a type, length and checksum. The payload depends on the
/// type value.
///
/// ObjFile represents the object file, and an OmfRecord one record.
///
///
/// Each type of record has a parser named after the record type in
/// the OMF specification, and an associated struct.
///  
/// 
use std::convert::TryFrom;
use std::fs;
use std::io;
use std::str;

use crate::error::Error as OmfError;
use crate::record::CommentClass;
use crate::record::RecordType;
use crate::record::Record;
use crate::record::RecordParser;


// aligments ('A' field of segdef acbp)
//
#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Align {
    Absolute = 0,
    Byte = 1,
    Word = 2,
    Para = 16,
    Page = 256,
    Dword = 4,
}

impl TryFrom<u8> for Align {
    type Error = OmfError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Align::Absolute),
            1 => Ok(Align::Byte),
            2 => Ok(Align::Word),
            3 => Ok(Align::Para),
            4 => Ok(Align::Page),
            5 => Ok(Align::Dword),
            _ => Err(OmfError::new(&format!("invalid align value ${:02x}", v)))
        }
    }
}

#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Combine {
    Private,
    Public,
    Stack,
    Common,
}

impl TryFrom<u8> for Combine {
    type Error = OmfError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Combine::Private),
            2 | 4 | 7 => Ok(Combine::Public),
            5 => Ok(Combine::Stack),
            6 => Ok(Combine::Common),
            _ => Err(OmfError::new(&format!("invalid combine value ${:02x}", v)))
        }
    }
}

const ACBP_BIG: u8 = 0x02;
const ACBP_USE32: u8 = 0x01;

// Represents an entire object file being parsed. Since object
// files are small compared to memory, the entire file is read
// in at the start of parsing.
//
pub struct ObjFile {
    data: Vec<u8>,
    next: usize,
}

// Compute the checkum of a potential record. The sum of the 
// record type, both bytes of the length, and the payload
// must be zero.
//
fn checksum(rectype: u8, lo: u16, hi: u16, data: &[u8]) -> bool {
    if data.len() == 0 {
        // corrupt - no checksum byte at all
        return false;
    }

    if data[data.len() - 1] == 0 {
        // no checksum if the checksum byte is 0
        return true;
    }

    let mut sum: u32 = 0;
    sum += rectype as u32;
    sum += lo as u32;
    sum += hi as u32;

    for x in data {
        sum += *x as u32
    }

    (sum & 0xff) == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_succeeds() {
        let b = checksum(1, 2, 3, &vec![(0x0100 - 6) as u8]);
        assert_eq!(b, true);
    }

    #[test]
    fn checksum_fails() {
        let b = checksum(1, 2, 4, &vec![(0x0100 - 6) as u8]);
        assert_eq!(b, false);
    }
}

impl<'a> ObjFile {
    // Create an object file, reset to the first record
    //
    pub fn new(data: Vec<u8>) -> ObjFile {
        ObjFile {
            data: data,
            next: 0,
        }
    }

    // Read an object file from disk
    //
    pub fn read(filename: &str) -> io::Result<ObjFile> {
        let data = fs::read(filename)?;
        Ok(ObjFile::new(data))
    }

    // Read the next object record and verify the checksum
    //
    pub fn next(&'a mut self) -> Result<Option<Record<'a>>, OmfError> {
        if self.next >= self.data.len() {
            Ok(None)
        } else {
            let left = self.data.len() - self.next;

            if left < 3 {
                return Err(OmfError::new("OMF record truncated"));
            }

            let rectype = self.data[self.next];
            let lo = self.data[self.next + 1] as u16;
            let hi = self.data[self.next + 2] as u16;

            let left = left - 3;
            let start = self.next + 3;
            let length = ((hi << 8) | lo) as usize;

            if left < length {
                return Err(OmfError::new("OMF record truncated"));
            }

            self.next = start + length;

            let slice = &self.data[start..self.next];
            if !checksum(rectype, lo, hi, slice) {
                return Err(OmfError::new("OMF record failed checksum"));
            }
            
            Ok(Some(Record{
                rectype: rectype.into(),
                data: &slice[0..slice.len()-1],
            }))
        }
    }

    // Read the next object record with no checksum (used by library
    // records).
    //
    pub fn next_no_checksum(&'a mut self) -> Result<Option<Record<'a>>, OmfError> {
        if self.next >= self.data.len() {
            Ok(None)
        } else {
            let left = self.data.len() - self.next;

            if left < 3 {
                return Err(OmfError::new("OMF record truncated"));
            }

            let rectype = self.data[self.next];
            let lo = self.data[self.next + 1] as u16;
            let hi = self.data[self.next + 2] as u16;

            let left = left - 3;
            let start = self.next + 3;
            let length = ((hi << 8) | lo) as usize;

            if left < length {
                return Err(OmfError::new("OMF record truncated"));
            }

            self.next = start + length;

            let slice = &self.data[start..self.next];
            Ok(Some(Record{
                rectype: rectype.into(),
                data: &slice[0..slice.len()-1],
            }))
        }
    }
}

#[cfg(test)]
mod objfile_tests {
    use super::*;

    #[test]
    fn test_object_file_parses_record() {
        let data = vec![0x80, 0x03, 0x00, 0x41, 0x42, 0xfa];
        let mut obj = ObjFile::new(data);

        if let Ok(Some(rec)) = obj.next() {
            if rec.rectype == RecordType::THeader {
                assert!(true);
            } else {
                assert!(false);
            }
            assert_eq!(rec.data.len(), 2);
            assert_eq!(rec.data, vec![0x41, 0x42]);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_object_file_errors_on_checksum() {
        let data = vec![0x80, 0x03, 0x00, 0x41, 0x42, 0xfb];
        let mut obj = ObjFile::new(data);

        if let Ok(Some(rec)) = obj.next() {
            if rec.rectype == RecordType::THeader {
                assert!(false);
            } else {
                assert!(false);
            }
        } else {
            assert!(true);
        }
    }

    #[test]
    fn test_object_file_errors_on_truncation() {
        let data = vec![0x80, 0x04, 0x00, 0x41, 0x42, 0xfa];
        let mut obj = ObjFile::new(data);

        if let Ok(Some(rec)) = obj.next() {
            if rec.rectype == RecordType::THeader {
                assert!(false);
            } else {
                assert!(false);
            }
        } else {
            assert!(true);
        }
    }

    #[test]
    fn test_object_file_detects_end() {
        let data = vec![0x80, 0x03, 0x00, 0x41, 0x42, 0xfa];
        let mut obj = ObjFile::new(data);

        if let Ok(Some(rec)) = obj.next() {
            if rec.rectype == RecordType::THeader {
                assert!(true);
            } else {
                assert!(false);
            }
            assert_eq!(rec.data.len(), 2);
            assert_eq!(rec.data, vec![0x41, 0x42]);
        } else {
            assert!(false);
        }

        if let Ok(None) = obj.next() {
            assert!(true);
        } else {
            assert!(false);
        }
    }


    #[test]
    fn test_object_file_handles_invalid_type() {
        // 0x81 is not a valid record type
        //
        let data = vec![0x81, 0x03, 0x00, 0x41, 0x42, 0xf9];
        let mut obj = ObjFile::new(data);

        if let Ok(Some(rec)) = obj.next() {
            let rectype = rec.rectype;

            if let RecordType::Unknown{typ: _} = rec.rectype {
                // by design, invalid record types have 'None'
                // here so we can still parse and skip records we
                // don't understand
                assert!(true)
            } else {
                assert!(false);
            }
        } else {
            assert!(false);
        }
    }
}

pub fn check_rectype(rec: &Record, rectype: RecordType, parser: &str) -> Result<RecordType, OmfError> {
    if rectype == rec.rectype {
        return Ok(rectype);
    }

    let type32 = u8::from(rectype) | 0x01;
    let type32 = type32.into();
    if type32 == rec.rectype {
        return Ok(type32);
    }        
        
    Err(OmfError::bad_rectype(rectype, parser))
}

// The THEADR record, which starts an object file
//
pub struct OmfTheadr {
    pub name: String,
}

impl OmfTheadr {
    // Parse a THEADR record
    //
    pub fn new(rec: &Record) -> Result<OmfTheadr, OmfError> {
        check_rectype(rec, RecordType::THeader, "THEADR")?;
        let mut parser = RecordParser::new(&rec);

        let name = parser.next_str()?;
        
        Ok(OmfTheadr{
            name: name,
        })
    }
}

// The MODEND record, which ends an object file and has optional
// entry point data
//
pub struct OmfModend {
    modtype: u8,
    enddata: Option<u8>,
    frame: Option<u16>,
    target: Option<u16>,
    displ: Option<u32>,
    is32: bool,
}

impl OmfModend {
    pub fn new(rec: &Record) -> Result<OmfModend, OmfError> {
        let rectype = check_rectype(rec, RecordType::ModEnd, "MODEND")?;
        let is32 = rectype.is32();
        let mut parser = RecordParser::new(&rec);

        let modtype = parser.next_byte()?;

        // TODO parse ending information

        Ok(OmfModend{
            modtype: modtype,
            enddata: None,
            frame: None,
            target: None,
            displ: None,
            is32: is32,
        })
    }
}


// The COMENT record. Contrary to the name, this is not 
// ignorable information. A COMENT record has a class
// which indicates the contents; this mechanism has been 
// used to extend the functionality of the original 
// defined set of record types.
//
pub struct Coment {
    pub comtype: u8,
    pub class: CommentClass,
}

impl Coment {
    // Parse a comment record header, which is just the
    // first two bytes.
    //
    pub fn new(parser: &mut RecordParser) -> Result<Coment, OmfError> {
        let comtype = parser.next_byte()?;
        let class = parser.next_byte()?.into();

        Ok(Coment{
            comtype: comtype,
            class: class,
        })
    }

    // Given a COMENT record, return the comment's class. 
    // The module user must use this to determine the appropriate
    // coment parser to use.
    //
    pub fn comclass(rec: &Record) -> Result<CommentClass, OmfError> {
        check_rectype(rec, RecordType::Comment, "COMENT")?;

        if rec.data.len() < 2 {
            Err(OmfError::truncated())
        } else {
            Ok(rec.data[1].into())
        }
    }
}

// A COMENT record which specifies the memory model.
//
pub struct OmfComentMemoryModel {
    pub com: Coment,
    pub model: String,
}

impl OmfComentMemoryModel {
    // Parse a COMENT library specification.
    //
    pub fn new(rec: &Record) -> Result<OmfComentMemoryModel, OmfError> {
        check_rectype(rec, RecordType::Comment, "COMENT")?;
        let mut parser = RecordParser::new(&rec);

        let com = Coment::new(&mut parser)?;
        if com.class != CommentClass::MemoryModel {
            println!("{:?}", rec);
            return Err(OmfError::bad_comclass(com.class, "COMENT_MEMORY_MODEL"))
        }

        let model = parser.rest_str()?;
        if model.len() == 0 {
            return Err(OmfError::truncated());
        }
        
        Ok(OmfComentMemoryModel{
            com: com,
            model: model,
        })
    }
}

// A COMENT record which specifies the DOS version (deprecated).
//
pub struct OmfComentDosVersion {
    pub com: Coment,
    pub version: String,
}

impl OmfComentDosVersion {
    // Parse a COMENT library specification.
    //
    pub fn new(rec: &Record) -> Result<OmfComentDosVersion, OmfError> {
        check_rectype(rec, RecordType::Comment, "COMENT")?;
        let mut parser = RecordParser::new(&rec);

        let com = Coment::new(&mut parser)?;
        if com.class != CommentClass::DosVersion {
            println!("{:?}", rec);
            return Err(OmfError::bad_comclass(com.class, "COMENT_DOS_VERSION"))
        }

        let version = parser.rest_str()?;
        if version.len() == 0 {
            return Err(OmfError::truncated());
        }
        
        Ok(OmfComentDosVersion{
            com: com,
            version: version,
        })
    }
}

// A COMENT record which specifies a library to link.
//
pub struct OmfComentLib {
    pub com: Coment,
    pub path: String,
}


impl OmfComentLib {
    // Parse a COMENT library specification.
    //
    pub fn new(rec: &Record) -> Result<OmfComentLib, OmfError> {
        check_rectype(rec, RecordType::Comment, "COMENT")?;
        let mut parser = RecordParser::new(&rec);

        let com = Coment::new(&mut parser)?;
        if com.class != CommentClass::DefaultLibrary {
            println!("{:?}", rec);
            return Err(OmfError::bad_comclass(com.class, "COMENT_LIB"))
        }

        let path = parser.rest_str()?;

        if path.len() == 0 {
            return Err(OmfError::truncated());
        }
        
        Ok(OmfComentLib{
            com: com,
            path: path,
        })
    }
}

// A list of names. Names are indexed starting at 1, across
// all LNAMES records. e.g. given these two records 
//
// LNAMES "a","b","c"
// LNAMES "foo"
//
// The names are indexed from other records as
// 1 "a"
// 2 "b"
// 3 "c"
// 4 "foo"
// 
pub struct OmfLnames {
    pub names: Vec<String>,
}

impl OmfLnames {
    // Parse an LNAMES record
    //
    pub fn new(rec: &Record) -> Result<OmfLnames, OmfError> {
        check_rectype(rec, RecordType::LNames, "LNAMES")?;

        let mut parser = RecordParser::new(&rec);
        let mut names = Vec::new();
        
        loop {
            if parser.end() {
                break;
            }

            names.push(parser.next_str()?);
        }

        Ok(OmfLnames{
            names: names,
        })
    }
}

// One logical segment definition
//
#[derive(Clone)]
pub struct OmfSegment {
    pub use32: bool,
    pub align: Align,       // alignment, parsed
    pub combine: Combine,   // combine, parsed
    pub frame: Option<u16>, // frame, if absolute
    pub offset: Option<u8>, // offset w/in frame
    pub length: u64,        // really 16 or 32 bits, we can represent a full 32-bit length 0x100000000
    pub name: usize,        // index into lnames
    pub class: usize,       // index into lnames
    pub overlay: usize,     // index into lnames
}

impl OmfSegment {
    pub fn empty() -> OmfSegment {
        OmfSegment {
            use32: false,
            align: Align::Byte,
            combine: Combine::Private,
            frame: None,
            offset: None,
            length: 0,
            name: 0,
            class: 0,
            overlay: 0,
        }
    }
}

// A SEGDEF record has multiple segment definitions
//
pub struct OmfSegdef {
    pub omfsegs: Vec<OmfSegment>,
}

impl OmfSegdef {
    // Parse a SEGDEF record
    //
    pub fn new(rec: &Record) -> Result<OmfSegdef, OmfError> {
        let rectype = check_rectype(rec, RecordType::SegDef, "SEGDEF")?;
        let is32 = rectype.is32();
        let mut parser = RecordParser::new(&rec);

        let mut segdefs = Vec::new();

        while !parser.end() {
            let acbp = parser.next_byte()?;
            let align: Align = ((acbp >> 5) & 7).try_into()?;

            let combine: Combine = ((acbp >> 2) & 7).try_into()?;
            
            let mut frame = Option::None;
            let mut offset = Option::None;

            if align == Align::Absolute {
                frame = Some(parser.next_uint(false)? as u16);
                offset = Some(parser.next_byte()?);
            }
            
            let given_length = parser.next_uint(is32)? as u64;

            let length = if acbp & ACBP_BIG != 0 {
                if given_length != 0 {
                    return Err(OmfError::new("segdef has BIG bit set but length is not zero."));
                }
                if is32 { 0x1_0000_0000 } else { 0x1_0000 }
            } else {
                given_length
            };
            
            let name = parser.next_index()?;
            let class = parser.next_index()?;
            let overlay = parser.next_index()?;

            segdefs.push(OmfSegment{
                use32: acbp & ACBP_USE32 != 0,
                align: align,
                combine: combine,
                frame: frame,
                offset: offset,
                length: length,
                class: class,
                name: name,
                overlay: overlay,
            })
        }

        Ok(OmfSegdef{
            omfsegs: segdefs,
        })
    }
}

pub struct OmfGrpdef {
    pub name: usize,
    pub segs: Vec<usize>,
}

impl OmfGrpdef {
    // Parse a SEGDEF record
    //
    pub fn new(rec: &Record) -> Result<OmfGrpdef, OmfError> {
        check_rectype(rec, RecordType::GrpDef, "GRPDEF")?;
        let mut parser = RecordParser::new(&rec);

        let name = parser.next_index()?;

        let mut grpdef = OmfGrpdef {
            name: name,
            segs: Vec::new(),
        };

        if parser.end() {
            return Err(OmfError::truncated());
        }

        while !parser.end() {
            let typindex = parser.next_byte()?;
            let seg = parser.next_index()?;
        
            // NB originally a group could contain other things, but any valid 
            // object file now has ff here meaning a segment index.
            if typindex != 0xff {
                return Err(OmfError::new("GRPDEF element must be a segment index"));
            }

            grpdef.segs.push(seg);
        }
        
        Ok(grpdef)
    }
}

pub struct OmfExtern {
    pub name: String,
    pub typindex: usize,
}

impl OmfExtern {
    fn new(name: String, typindex: usize) -> OmfExtern {
        OmfExtern {
            name: name,
            typindex: typindex,
        }
    }
}

pub struct OmfExtdef {
    pub externs: Vec<OmfExtern>,
}

impl OmfExtdef {
    // Parse an EXTDEF record
    //
    pub fn new(rec: &Record) -> Result<OmfExtdef, OmfError> {
        check_rectype(rec, RecordType::ExtDef, "EXTDEF")?;
        let mut parser = RecordParser::new(&rec);
        let mut extdef = OmfExtdef {
            externs: Vec::new(),
        }; 

        while !parser.end() {
            let name = parser.next_str()?;
            let typindex = parser.next_index()?;
            extdef.externs.push(OmfExtern::new(name, typindex));
        }        

        Ok(extdef)
    }
}

pub struct OmfPublic {
    pub name: String,
    pub offset: u32,
    pub typindex: usize,
}

impl OmfPublic {
    fn new(name: String, offset: u32, typindex: usize) -> OmfPublic {
        OmfPublic {
            name: name,
            offset: offset,
            typindex: typindex,
        }
    }
}

pub struct OmfPubdef {
    pub base_group: Option<usize>,
    pub base_seg: Option<usize>,
    pub base_frame: Option<u16>,
    pub publics: Vec<OmfPublic>,
}


impl OmfPubdef {
    // Parse a PUBDEF record
    //
    pub fn new(rec: &Record) -> Result<OmfPubdef, OmfError> {
        let rectype = check_rectype(rec, RecordType::PubDef, "PUBDEF")?;
        let is32 = rectype.is32();
        let mut parser = RecordParser::new(&rec);

        let group = parser.next_index()?;
        let seg = parser.next_index()?;
        let frame = if seg == 0 {
            Some(parser.next_uint(false)? as u16)
        } else {
            None
        };


        let mut pubdef = OmfPubdef {
            base_group: if group != 0 { Some(group) } else { None },
            base_seg: if seg != 0 { Some(seg) } else { None },
            base_frame: frame,
            publics: vec![],
        };

        if parser.end() {
            return Err(OmfError::new("PUBDEF contains no publics"));
        }

        while !parser.end() {
            let name = parser.next_str()?;
            let offset = parser.next_uint(is32)?;
            let typindex = parser.next_index()?;

            pubdef.publics.push(OmfPublic::new(name, offset, typindex));
        }

        Ok(pubdef)
    }
}

pub struct OmfLExtdef {
    pub externs: Vec<OmfExtern>,
}

impl OmfLExtdef {
    // Parse an LEXTDEF record
    //
    pub fn new(rec: &Record) -> Result<OmfLExtdef, OmfError> {
        check_rectype(rec, RecordType::LExtDef, "LEXTDEF")?;
        let mut parser = RecordParser::new(&rec);
        let mut extdef = OmfLExtdef {
            externs: Vec::new(),
        }; 

        while !parser.end() {
            let name = parser.next_str()?;
            let typindex = parser.next_index()?;
            extdef.externs.push(OmfExtern::new(name, typindex));
        }        

        Ok(extdef)
    }
}

#[cfg(test)]
mod omfrec_tests {
    use super::*;

    
    //
    // THeader
    //

    #[test]
    fn test_theader_succeeds() {
        let rec = Record{
            rectype: RecordType::THeader,
            data: &vec![0x03, 0x41, 0x42, 0x43],
        };

        if let Ok(rec) = OmfTheadr::new(&rec) {
            assert_eq!(rec.name, "ABC");    
        } else {
            assert!(false, "parse of valid THEADR failed");
        }
    }

    #[test]
    fn test_theader_errors_on_bad_type() {
        let rec = Record{
            rectype: RecordType::SegDef,
            data: &vec![0x03, 0x41, 0x42, 0x43],
        };

        if let Ok(_) = OmfTheadr::new(&rec) {
            assert!(false, "parse suceeded with invalid record type");    
        }
    }
    
    //
    // ModEnd
    //

    #[test]
    fn test_modend_no_start_succeeds() {
        let rec = Record{
            rectype: RecordType::ModEnd,
            data: &vec![0x00],  
        };

        if let Ok(rec) = OmfModend::new(&rec) {
            assert_eq!(rec.modtype, 0x00);
            assert_eq!(rec.enddata, None);
            assert_eq!(rec.frame, None);
            assert_eq!(rec.target, None);
            assert_eq!(rec.displ, None);
            assert_eq!(rec.is32, false);
        } else {
            assert!(false, "parse of valid MODEND fails");
        }
    }

    #[test]
    fn test_modend_errors_on_bad_type() {
        let rec = Record{
            rectype: RecordType::SegDef,
            data: &vec![0x00],
        };

        if let Ok(_) = OmfModend::new(&rec) {
            assert!(false, "parse of invalid MODEND succeeded")
        }
    }

    //
    // Coment
    //

    #[test]
    fn test_coment_type_and_class_parsed_correctly() {
        let rec = Record{
            rectype: RecordType::Comment,
            data: &vec![0x80, 0x30],
        };

        let mut parser = RecordParser::new(&rec);

        if let Ok(coment) = Coment::new(&mut parser) {
            assert_eq!(coment.comtype, 0x80);
            assert_eq!(coment.class, CommentClass::Unknown{ typ: 0x30 });
        } else {
            assert!(false, "parse of valid COMENT failed");
        }
    }

    #[test]
    fn test_coment_fails_on_bounds() {
        let rec = Record{
            rectype: RecordType::Comment,
            data: &vec![0x80],
        };

        let mut parser = RecordParser::new(&rec);

        if let Ok(_) = Coment::new(&mut parser) {
            assert!(false, "parsing truncated COMENT record succeeded");
        }
    }

    #[test]
    fn test_parse_coment_lib_succeeds() {
        let rec = Record{
            rectype: RecordType::Comment,
            data: &vec![0x80, 0x9f, 0x41, 0x42, 0x43],
        };

        if let Ok(lib) = OmfComentLib::new(&rec) {
            assert_eq!(lib.com.comtype, 0x80);
            assert_eq!(lib.com.class, CommentClass::DefaultLibrary);
            assert_eq!(lib.path, "ABC".to_string());
        } else {
            assert!(false, "parsing valid COMENT LIB failed");
        }
    }

    #[test]
    fn test_parse_truncated_coment_lib_fails() {
        let rec = Record{
            rectype: RecordType::Comment,
            data: &vec![0x80, 0x9f],
        };

        if let Ok(_) = OmfComentLib::new(&rec) {
            assert!(false, "parsing truncated COMENT LIB succeeded");
        } 
    }

    #[test]
    fn test_parse_coment_memory_model_succeeds() {
        let rec = Record{
            rectype: RecordType::Comment,
            data: &vec![0x80, 0x9d, 0x33, 0x4f, 0x73],
        };

        if let Ok(lib) = OmfComentMemoryModel::new(&rec) {
            assert_eq!(lib.com.comtype, 0x80);
            assert_eq!(lib.com.class, CommentClass::MemoryModel);
            assert_eq!(lib.model, "3Os".to_string());
        } else {
            assert!(false, "parsing valid COMENT memory model failed");
        }
    }

    #[test]
    fn test_parse_truncated_coment_memory_model_fails() {
        let rec = Record{
            rectype: RecordType::Comment,
            data: &vec![0x80, 0x9d],
        };

        if let Ok(_) = OmfComentMemoryModel::new(&rec) {
            assert!(false, "parsing truncated COMENT memory model succeeded");
        } 
    }

    #[test]
    fn test_parse_coment_dos_version_succeeds() {
        let rec = Record{
            rectype: RecordType::Comment,
            data: &vec![0x80, 0x9c, 0x33, 0x30],
        };

        if let Ok(lib) = OmfComentDosVersion::new(&rec) {
            assert_eq!(lib.com.comtype, 0x80);
            assert_eq!(lib.com.class, CommentClass::DosVersion);
            assert_eq!(lib.version, "30".to_string());
        } else {
            assert!(false, "parsing valid COMENT DOS version failed");
        }
    }

    #[test]
    fn test_parse_truncated_coment_dos_version_fails() {
        let rec = Record{
            rectype: RecordType::Comment,
            data: &vec![0x80, 0x9c],
        };

        if let Ok(_) = OmfComentDosVersion::new(&rec) {
            assert!(false, "parsing truncated COMENT DOS version succeeded");
        } 
    }

    #[test]
    fn test_lnames_parses_names() {
        let rec = Record{
            rectype: RecordType::LNames,
            data: &vec![0x03, 0x41, 0x42, 0x43, 0x03, 0x44, 0x45, 0x46],
        };

        let lnames = OmfLnames::new(&rec);
        if let Ok(names) = lnames {
            assert_eq!(names.names, vec!["ABC".to_string(), "DEF".to_string()]);
        } else {
            assert!(false, "failed to parse valid lnames data");
        }
    }

    #[test]
    fn test_lnames_errors_on_bad_type() {
        let rec = Record{
            rectype: RecordType::SegDef,
            data: &vec![0x03, 0x41, 0x42, 0x43, 0x03, 0x44, 0x45, 0x46],
        };

        let lnames = OmfLnames::new(&rec);
        if let Ok(_) = lnames {
            assert!(false, "succeeded parsing lnames with bad record type");
        }
    }

    #[test]
    fn test_lnames_errors_on_truncation() {
        let rec = Record{
            rectype: RecordType::LNames,
            data: &vec![0x03, 0x41, 0x42, 0x43, 0x03, 0x44, 0x45],
        };

        let lnames = OmfLnames::new(&rec);
        if let Ok(_) = lnames {
            assert!(false, "succeeded parsing lnames with truncated data");
        }
    }

    #[test]
    fn test_segdef_parses_segdef() {
        let rec = Record{
            rectype: RecordType::SegDef,
            data: &vec![
                0b00101000,     // ACBP: byte aligned, public
                0x23, 0x01,     // length 0x0123
                4,              // segment name index
                5,              // class name index
                6,              // overlay name index
            ],
        };

        let segdef = OmfSegdef::new(&rec);
        if let Ok(segdef) = segdef {
            assert_eq!(segdef.omfsegs.len(), 1);
            let def = &segdef.omfsegs[0];

            assert_eq!(def.use32, false);
            assert_eq!(def.align, Align::Byte);
            assert_eq!(def.combine, Combine::Public);
            if let Some(_) = def.frame { assert!(false, "segment should not have a frame"); }
            if let Some(_) = def.offset { assert!(false, "segment should not have an offset"); }
            assert_eq!(def.length, 0x0123);
            assert_eq!(def.name, 4);
            assert_eq!(def.class, 5);
            assert_eq!(def.overlay, 6);
        } else {
            assert!(false, "failed to parse valid segdef");
        }
    }

    #[test]
    fn test_segdef_parses_mutliple_defs() {
        let rec = Record{
            rectype: RecordType::SegDef,
            data: &vec![
                0b00101000,     // ACBP: byte aligned, public
                0x23, 0x01,     // length 0x0123
                4,              // segment name index
                5,              // class name index
                6,              // overlay name index
                0b01000000,     // ACBP: word aligned, private
                0xdc, 0x00,     // length 0x00dc
                7,              // segment name index
                8,              // class name index
                9,              // overlay name index
            ],
        };

        let segdef = OmfSegdef::new(&rec);
        if let Ok(segdef) = segdef {
            assert_eq!(segdef.omfsegs.len(), 2);
            let def = &segdef.omfsegs[0];

            assert_eq!(def.use32, false);
            assert_eq!(def.align, Align::Byte);
            assert_eq!(def.combine, Combine::Public);
            if let Some(_) = def.frame { assert!(false, "segment should not have a frame"); }
            if let Some(_) = def.offset { assert!(false, "segment should not have an offset"); }
            assert_eq!(def.length, 0x0123);
            assert_eq!(def.name, 4);
            assert_eq!(def.class, 5);
            assert_eq!(def.overlay, 6);

            let def = &segdef.omfsegs[1];

            assert_eq!(def.use32, false);
            assert_eq!(def.align, Align::Word);
            assert_eq!(def.combine, Combine::Private);
            if let Some(_) = def.frame { assert!(false, "segment should not have a frame"); }
            if let Some(_) = def.offset { assert!(false, "segment should not have an offset"); }
            assert_eq!(def.length, 0x00dc);
            assert_eq!(def.name, 7);
            assert_eq!(def.class, 8);
            assert_eq!(def.overlay, 9);
        } else {
            assert!(false, "failed to parse valid segdef");
        }
    }

    #[test]
    fn test_segdef_parses_absolute_segment() {
        let rec = Record{
            rectype: RecordType::SegDef,
            data: &vec![
                0b00010100,     // ACBP: absolute, stack
                0x56, 0x78,     // frame
                0xdd,           // offset
                0x23, 0x01,     // length 0x0123
                4,              // segment name index
                5,              // class name index
                6,              // overlay name index
            ],
        };

        let segdef = OmfSegdef::new(&rec);
        if let Ok(segdef) = segdef {
            assert_eq!(segdef.omfsegs.len(), 1);
            let def = &segdef.omfsegs[0];

            assert_eq!(def.align, Align::Absolute);
            assert_eq!(def.combine, Combine::Stack);

            if let Some(frame) = def.frame {
                assert_eq!(frame, 0x7856);
            } else { 
                assert!(false, "segment should have a frame"); 
            }

            if let Some(offset) = def.offset {
                assert_eq!(offset, 0xdd);
            } else { 
                assert!(false, "segment should not have an offset"); 
            }

            assert_eq!(def.use32, false);
            assert_eq!(def.length, 0x0123);
            assert_eq!(def.name, 4);
            assert_eq!(def.class, 5);
            assert_eq!(def.overlay, 6);
        } else {
            assert!(false, "failed to parse valid segdef");
        }
    }

    #[test]
    fn test_segdef_validates_alignment() {
        let rec = Record{
            rectype: RecordType::SegDef,
            data: &vec![
                0b11101000,     // ACBP: invalid alignment, public
                0x23, 0x01,     // length 0x0123
                4,              // segment name index
                5,              // class name index
                6,              // overlay name index
            ],
        };

        let segdef = OmfSegdef::new(&rec);
        if let Ok(_) = segdef {
            assert!(false, "did not fail on invalid alignment");
        } 
    }

    #[test]
    fn test_segdef_validates_combination() {
        let rec = Record{
            rectype: RecordType::SegDef,
            data: &vec![
                0b00100100,     // ACBP: byte alignment, invalid combination
                0x23, 0x01,     // length 0x0123
                4,              // segment name index
                5,              // class name index
                6,              // overlay name index
            ],
        };

        let segdef = OmfSegdef::new(&rec);
        if let Ok(_) = segdef {
            assert!(false, "did not fail on invalid combination");
        } 
    }

    #[test]
    fn test_segdef_fails_on_bounds() {
        let rec = Record{
            rectype: RecordType::SegDef,
            data: &vec![
                0b00100100,     // ACBP: byte alignment, invalid combination
                0x23, 0x01,     // length 0x0123
                4,              // segment name index
                5,              // class name index
                // missing overlay
            ],
        };

        let segdef = OmfSegdef::new(&rec);
        if let Ok(_) = segdef {
            assert!(false, "did not fail on truncated record");
        } 
    }

    #[test]
    fn test_segdef_fails_on_bad_type() {
        let rec = Record{
            rectype: RecordType::LNames,
            data: &vec![
                0b00100100,     // ACBP: byte alignment, invalid combination
                0x23, 0x01,     // length 0x0123
                4,              // segment name index
                5,              // class name index
                6,              // overlay name index
            ],
        };

        let segdef = OmfSegdef::new(&rec);
        if let Ok(_) = segdef {
            assert!(false, "did not fail on truncated record");
        } 
    }


    #[test]
    fn test_segdef_handles_big_bit() {
        let rec = Record{
            rectype: RecordType::SegDef,
            data: &vec![
                0b00101010,     // ACBP: byte alignment, public, big
                0x00, 0x00,     // length 0
                4,              // segment name index
                5,              // class name index
                6,              // overlay name index
            ],
        };

        if let Ok(segdef) = OmfSegdef::new(&rec) {
            assert_eq!(segdef.omfsegs.len(), 1);
            let def = &segdef.omfsegs[0];

            assert_eq!(def.align, Align::Byte);
            assert_eq!(def.combine, Combine::Public);
            if let Some(_) = def.frame { assert!(false, "segment should not have a frame"); }
            if let Some(_) = def.offset { assert!(false, "segment should not have an offset"); }
            assert_eq!(def.length, 0x10000);
            assert_eq!(def.name, 4);
            assert_eq!(def.class, 5);
            assert_eq!(def.overlay, 6);
        } else {
            assert!(false, "failed to parse valid segdef");
        }
    }

    #[test]
    fn test_segdef_handles_use32_bit() {
        let rec = Record{
            rectype: RecordType::SegDef,
            data: &vec![
                0b00101001,     // ACBP: byte alignment, public, use32
                0x12, 0x00,     // length 0x12
                4,              // segment name index
                5,              // class name index
                6,              // overlay name index
            ],
        };

        if let Ok(segdef) = OmfSegdef::new(&rec) {
            assert_eq!(segdef.omfsegs.len(), 1);
            let def = &segdef.omfsegs[0];

            assert_eq!(def.use32, true);
            assert_eq!(def.align, Align::Byte);
            assert_eq!(def.combine, Combine::Public);
            if let Some(_) = def.frame { assert!(false, "segment should not have a frame"); }
            if let Some(_) = def.offset { assert!(false, "segment should not have an offset"); }
            assert_eq!(def.length, 0x12);
            assert_eq!(def.name, 4);
            assert_eq!(def.class, 5);
            assert_eq!(def.overlay, 6);
        } else {
            assert!(false, "failed to parse valid segdef");
        }
    }

    #[test]
    fn test_segdef_parses_32_bit_length() {
        let rec = Record{
            rectype: RecordType::SegDef32,
            data: &vec![
                0b00101000,     // ACBP: byte aligned, public
                0x78, 0x56, 0x23, 0x01,     // length 0x01235678
                4,              // segment name index
                5,              // class name index
                6,              // overlay name index
            ],
        };

        let segdef = OmfSegdef::new(&rec);
        if let Ok(segdef) = segdef {
            assert_eq!(segdef.omfsegs.len(), 1);
            let def = &segdef.omfsegs[0];

            assert_eq!(def.use32, false);
            assert_eq!(def.align, Align::Byte);
            assert_eq!(def.combine, Combine::Public);
            if let Some(_) = def.frame { assert!(false, "segment should not have a frame"); }
            if let Some(_) = def.offset { assert!(false, "segment should not have an offset"); }
            assert_eq!(def.length, 0x01235678);
            assert_eq!(def.name, 4);
            assert_eq!(def.class, 5);
            assert_eq!(def.overlay, 6);
        } else {
            assert!(false, "failed to parse valid segdef");
        }
    }

    #[test]
    fn test_segdef_parses_32_bit_big_bit() {
        let rec = Record{
            rectype: RecordType::SegDef32,
            data: &vec![
                0b00101010,     // ACBP: byte aligned, public, big
                0, 0, 0, 0,     // length 0
                4,              // segment name index
                5,              // class name index
                6,              // overlay name index
            ],
        };

        let segdef = OmfSegdef::new(&rec);
        if let Ok(segdef) = segdef {
            assert_eq!(segdef.omfsegs.len(), 1);
            let def = &segdef.omfsegs[0];

            assert_eq!(def.use32, false);
            assert_eq!(def.align, Align::Byte);
            assert_eq!(def.combine, Combine::Public);
            if let Some(_) = def.frame { assert!(false, "segment should not have a frame"); }
            if let Some(_) = def.offset { assert!(false, "segment should not have an offset"); }
            assert_eq!(def.length, 1u64 << 32);
            assert_eq!(def.name, 4);
            assert_eq!(def.class, 5);
            assert_eq!(def.overlay, 6);
        } else {
            assert!(false, "failed to parse valid segdef");
        }
    }

    //
    // Grpdef
    //
    #[test]
    fn test_grpdef_succeeds() {
        let rec = Record {
            rectype: RecordType::GrpDef,
            data: &vec![0x02, 0xff, 0x01, 0xff, 0x02],
        };

        if let Ok(rec) = OmfGrpdef::new(&rec) {
            assert_eq!(rec.name, 2);
            assert_eq!(rec.segs.len(), 2);
            assert_eq!(rec.segs[0], 1);
            assert_eq!(rec.segs[1], 2);
        } else {
            assert!(false, "failed to parse valid grpdef");
        }
    }

    #[test]
    fn test_grpdef_errors_on_bad_type() {
        let rec = Record {
            rectype: RecordType::SegDef,
            data: &vec![0x02, 0xff, 0x01, 0xff, 0x02],
        };

        assert!(OmfGrpdef::new(&rec).is_err());
    }

    #[test]
    fn test_grpdef_errors_on_bad_element_type() {
        let rec = Record {
            rectype: RecordType::GrpDef,
            data: &vec![0x02, 0xfe, 0x01],
        };

        assert!(OmfGrpdef::new(&rec).is_err());
    }

    #[test]
    fn test_grpdef_errors_on_truncation() {
        let rec = Record {
            rectype: RecordType::GrpDef,
            data: &vec![0x02, 0xff, 0x01, 0xff],
        };

        assert!(OmfGrpdef::new(&rec).is_err());
    }

    #[test]
    fn test_grpdef_errors_on_no_segments() {
        let rec = Record {
            rectype: RecordType::GrpDef,
            data: &vec![0x02],
        };

        assert!(OmfGrpdef::new(&rec).is_err());
    }

    //
    // Extdef
    //

    #[test]
    fn test_extdef_succeeds() {
        let rec = Record {
            rectype: RecordType::ExtDef,
            data: &vec![
                0x03, 0x41, 0x42, 0x43, 0x01,
                0x04, 0x44, 0x45, 0x46, 0x47, 0x81, 0x02,
            ],
        };

        if let Ok(rec) = OmfExtdef::new(&rec) {
            assert_eq!(rec.externs.len(), 2);
            
            let ext = &rec.externs[0];
            assert_eq!(ext.name, "ABC");
            assert_eq!(ext.typindex, 1);

            let ext = &rec.externs[1];
            assert_eq!(ext.name, "DEFG");
            assert_eq!(ext.typindex, 0x0102);
        } else {
            assert!(false, "parse of valid EXTDEF failed");
        }
    }

    #[test]
    fn test_extdef_errors_on_bad_type() {
        let rec = Record{
            rectype: RecordType::SegDef,
            data: &vec![
                0x03, 0x41, 0x42, 0x43, 0x01,
            ],
        };

        if let Ok(_) = OmfExtdef::new(&rec) {
            assert!(false, "parse suceeded with invalid record type");    
        }
    }
    
    #[test]
    fn test_extdef_errors_on_truncation() {
        let rec = Record{
            rectype: RecordType::ExtDef,
            data: &vec![
                0x03, 0x41, 0x42, 0x43,
            ],
        };

        if let Ok(_) = OmfExtdef::new(&rec) {
            assert!(false, "parse suceeded with truncated record");    
        }
    }

    //
    // Pubdef
    //
    #[test]
    fn test_pubdef_succeeds() {
        let rec = Record{
            rectype: RecordType::PubDef,
            data: &vec![
                0x01,       // base group
                0x02,       // base segment
                0x03, 0x41, 0x42, 0x43, 0x34, 0x12, 0x00,
                0x03, 0x44, 0x45, 0x46, 0x78, 0x56, 0x81, 0x02,
            ],
        };

        if let Ok(rec) = OmfPubdef::new(&rec) {
            assert_eq!(rec.base_group, Some(1));
            assert_eq!(rec.base_seg, Some(2));
            assert!(rec.base_frame.is_none());
            
            assert_eq!(rec.publics.len(), 2);
            
            let public = &rec.publics[0];
            assert_eq!(public.name, "ABC");
            assert_eq!(public.offset, 0x1234);
            assert_eq!(public.typindex, 0);

            let public = &rec.publics[1];
            assert_eq!(public.name, "DEF");
            assert_eq!(public.offset, 0x5678);
            assert_eq!(public.typindex, 0x0102);

        } else {
            assert!(false, "parse of valid PUBDEF failed");
        }
    }

    #[test]
    fn test_pubdef_fails_on_truncated_record() {
        let rec = Record{
            rectype: RecordType::PubDef,
            data: &vec![
                0x01,       // base group
            ],
        };

        assert!(!OmfPubdef::new(&rec).is_ok());        
    }

    #[test]
    fn test_pubdef_fails_on_no_publics() {
        let rec = Record{
            rectype: RecordType::PubDef,
            data: &vec![
                0x01,       // base group
                0x02,       // base segment
            ],
        };

        assert!(!OmfPubdef::new(&rec).is_ok());        
    }

    #[test]
    fn test_pubdef_fails_on_bad_type() {
        let rec = Record{
            rectype: RecordType::ExtDef,
            data: &vec![
                0x01,       // base group
                0x02,       // base segment
                0x03, 0x41, 0x42, 0x43, 0x34, 0x12, 0x00,
            ],
        };

        assert!(!OmfPubdef::new(&rec).is_ok());        
    }

    #[test]
    fn test_pubdef_parses_absolute_frame() {
        let rec = Record{
            rectype: RecordType::PubDef,
            data: &vec![
                0x00,       // base group
                0x00,       // base segment
                0xaa, 0x55, // absolute frame
                0x03, 0x41, 0x42, 0x43, 0x34, 0x12, 0x00,
            ],
        };

        if let Ok(rec) = OmfPubdef::new(&rec) {
            assert_eq!(rec.base_group, None);
            assert_eq!(rec.base_seg, None);
            assert!(!rec.base_frame.is_none());
            assert_eq!(rec.base_frame.unwrap(), 0x55aa);
            
            assert_eq!(rec.publics.len(), 1);
            
            let public = &rec.publics[0];
            assert_eq!(public.name, "ABC");
            assert_eq!(public.offset, 0x1234);
            assert_eq!(public.typindex, 0);

        } else {
            assert!(false, "parse of valid PUBDEF failed");
        }
    }

    #[test]
    fn test_pubdef_parses_32_bit_offset() {
        let rec = Record{
            rectype: RecordType::PubDef32,
            data: &vec![
                0x01,       // base group
                0x02,       // base segment
                0x03, 0x41, 0x42, 0x43, 0x78, 0x56, 0x34, 0x12, 0x3c,
            ],
        };

        if let Ok(rec) = OmfPubdef::new(&rec) {
            assert_eq!(rec.base_group, Some(1));
            assert_eq!(rec.base_seg, Some(2));
            assert!(rec.base_frame.is_none());
            
            assert_eq!(rec.publics.len(), 1);
            
            let public = &rec.publics[0];
            assert_eq!(public.name, "ABC");
            assert_eq!(public.offset, 0x12345678);
            assert_eq!(public.typindex, 0x3c);

        } else {
            assert!(false, "parse of valid PUBDEF failed");
        }
    }
}

