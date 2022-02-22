use crate::error::Error as ObjError;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum FrameMethod {
    Segdef,
    Grpdef,
    Extdef,
    PreviousDataRecord,
    Target,
}

impl TryFrom<u8> for FrameMethod {
    type Error = ObjError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(FrameMethod::Segdef),
            1 => Ok(FrameMethod::Grpdef),
            2 => Ok(FrameMethod::Extdef),
            4 => Ok(FrameMethod::PreviousDataRecord),
            5 => Ok(FrameMethod::Target),

            val => Err(ObjError::new(&format!("invalid frame method ${:02x}", val))),
        }
    }
}

impl FrameMethod {
    fn has_datum(&self) -> bool {
        *self == FrameMethod::Segdef ||
        *self == FrameMethod::Grpdef || 
        *self == FrameMethod::Extdef
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum TargetMethod {
    Segdef,
    Grpdef,
    Extdef,
    SegdefNoDisplacement,
    GrpdefNoDisplacement,
    ExtdefNoDisplacement,
}

impl TryFrom<u8> for TargetMethod {
    type Error = ObjError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(TargetMethod::Segdef),
            1 => Ok(TargetMethod::Grpdef),
            2 => Ok(TargetMethod::Extdef),
            4 => Ok(TargetMethod::SegdefNoDisplacement),
            5 => Ok(TargetMethod::GrpdefNoDisplacement),
            6 => Ok(TargetMethod::ExtdefNoDisplacement),

            val => Err(ObjError::new(&format!("invalid target method ${:02x}", val))),
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct StartAddress {
    pub fix_data: u8,
    pub frame_datum: Option<usize>,
    pub target_datum: Option<usize>,
    pub target_disp: Option<u32>,
}

impl StartAddress {
    pub fn fthread(&self) -> bool {
        (self.fix_data & 0x80) != 0
    }

    pub fn fmethod(&self) -> Result<Option<FrameMethod>, ObjError> {
        Ok(if self.fthread() { None } else {
            let method = ((self.fix_data >> 4) & 7).try_into()?;
            Some(method)
        })
    }

    pub fn fthreadno(&self) -> Option<usize> {
        if self.fthread() {
            Some(((self.fix_data >> 4) & 7) as usize)
        } else { 
            None
        }
    }

    pub fn tthread(&self) -> bool {
        (self.fix_data & 0x08) != 0
    }

    pub fn tmethod(&self) -> Result<Option<TargetMethod>, ObjError> {
        Ok(if self.tthread() { None } else {
            let method = (self.fix_data & 7).try_into()?;
            Some(method)
        })
    }

    pub fn tthreadno(&self) -> Option<usize> {
        if self.tthread() {
            Some((self.fix_data & 7) as usize)
        } else { 
            None
        }
    }
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Align {
    Absolute,
    Byte,
    Word,
    Paragraph,
    Page,
    Dword,
}

impl TryFrom<u8> for Align {
    type Error = ObjError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Align::Absolute),
            1 => Ok(Align::Byte),
            2 => Ok(Align::Word),
            3 => Ok(Align::Paragraph),
            4 => Ok(Align::Page),
            5 => Ok(Align::Dword),

            val => Err(ObjError::new(&format!("invalid align ${:02x}", val))),
        }
    }
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Combine {
    Private,
    Public,
    Stack,
    Common,
}

impl TryFrom<u8> for Combine {
    type Error = ObjError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Combine::Private),
            2|4|7 => Ok(Combine::Public),
            5 => Ok(Combine::Stack),
            6 => Ok(Combine::Common),

            val => Err(ObjError::new(&format!("invalid combine ${:02x}", val))),
        }
    }
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct AbsoluteSeg {
    pub frame: u16,
    pub offset: u8,
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct Segdef {
    pub align: Align,
    pub combine: Combine,
    pub use32: bool,
    pub abs: Option<AbsoluteSeg>,
    pub length: u64,
    pub class: Option<usize>,
    pub name: Option<usize>,
    pub overlay: Option<usize>,
}

impl Segdef {
    pub fn empty() -> Segdef {
        Segdef {
            align: Align::Byte,
            combine: Combine::Public,
            use32: false,
            abs: None,
            length: 0,
            class: None,
            name: None,
            overlay: None,
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Extern {
    pub name: String,
    pub typeidx: usize,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct Public {
    pub name: String,
    pub offset: u32,
    pub typeidx: usize,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct ComentHeader {
    pub comtype: u8,
    pub comclass: u8,
}

impl ComentHeader {
    pub fn nopurge(&self) -> bool {
        (self.comtype & 0x80) != 0
    }

    pub fn nolist(&self) -> bool {
        (self.comtype & 0x40) != 0
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Coment {
    Unknown,
    Translator{ text: String },
    MemoryModel{ text: String },
    NewOMF{ text: String },
    DefaultLibrary{ name: String },
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum BakpatLocation {
    Byte,
    Word,
    Dword,
}

impl TryFrom<u8> for BakpatLocation {
    type Error = ObjError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(BakpatLocation::Byte),
            1 => Ok(BakpatLocation::Word),
            2|9 => Ok(BakpatLocation::Dword),

            val => Err(ObjError::new(&format!("invalid BAKPAT location ${:02x}", val))),
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct BakpatFixup {
    pub offset: u32,
    pub value: u32,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Record {
    None,
    Unknown{ rectype: u8 },

    THEADR{ name: String },
    MODEND{ main: bool, start_address: Option<StartAddress> },
    LNAMES{ names: Vec<String> },
    SEGDEF{ segs: Vec<Segdef> },
    GRPDEF{ name: usize, segs: Vec<usize> },
    EXTDEF{ externs: Vec<Extern> },
    PUBDEF{ group: Option<usize>, seg: Option<usize>, frame: Option<u16>, publics: Vec<Public> },
    COMENT{ header: ComentHeader, coment: Coment },
    LEDATA{ seg: usize, offset: u32, data: Vec<u8> },
    BAKPAT{ seg: usize, location: BakpatLocation, fixups: Vec<BakpatFixup> },
}

pub struct Parser {
    obj: Vec<u8>,
    start: usize,
    ptr: usize,
    next: usize,
}

impl Parser {
    pub fn new(obj: Vec<u8>) -> Parser {
        Parser{ obj, start: 0, ptr: 0, next: 0 }
    }

    fn err(&self, err: &str) -> ObjError {
        ObjError::with_offset(err, self.start)
    }

    fn endrec(&self) -> usize {
        // record end does not include checksum byte
        self.next - 1
    }

    fn uint(data: &[u8]) -> usize {
        let bytes = data.len();
        let mut value: usize = 0;
    
        for i in 1..bytes+1 {
            let byte = data[bytes - i] as usize;
            value = (value << 8) | byte;
        }
    
        value
    }
    
    fn next_uint(&mut self, size: usize) -> Result<usize, ObjError> {
        if self.ptr + size > self.endrec() {
            Err(self.err("next_uint: record is truncated"))
        } else {
            let value = Self::uint(&self.obj[self.ptr..self.ptr+size]);
            self.ptr += size;
            Ok(value)
        }
    }

    fn next_str(&mut self) -> Result<String, ObjError> {
        if self.ptr >= self.endrec() {
            Err(self.err("next_str: no length byte"))
        } else {
            let len = self.obj[self.ptr] as usize;
            self.ptr += 1;
    
            if self.ptr + len > self.obj.len() {
                Err(self.err("next_str: string is truncated"))
            } else {
                let s = &self.obj[self.ptr..self.ptr+len];
                self.ptr += len;
        
                String::from_utf8(s.to_vec()).map_err(|err| self.err(&format!("{:x?}", err)))
            }
        }
    }

    fn rest_str(&mut self) -> Result<String, ObjError> {
        let bytes = &self.obj[self.ptr..self.endrec()];
        self.ptr = self.endrec();
        String::from_utf8(bytes.to_vec()).map_err(|err| self.err(&format!("{:x?}", err)))
    }

    fn next_index(&mut self) -> Result<usize, ObjError> {
        let index = self.next_uint(1)?;

        if index < 0x80 {
            Ok(index)
        } else {
            Ok(
                ((index & 0x7f) << 8) | self.next_uint(1)?
            )
        }
    }

    fn next_opt_index(&mut self) -> Result<Option<usize>, ObjError> {
        Ok(match self.next_index()? {
            0 => None,
            index => Some(index),
        })
    }

    fn checksum(bytes: &[u8]) -> bool {
        if *bytes.last().unwrap() == 0 {
            true
        } else {
            let mut sum = 0;
            for byte in  bytes {
                sum += *byte as usize;
            }

            (sum & 0xff) == 0
        }
    }

    fn modend(&mut self, is32: bool) -> Result<Record, ObjError> {
        let modtype = self.next_uint(1)?;

        let main = (modtype & 0x80) != 0;
        let has_start = (modtype & 0x40) != 0;

        // NB the spec claims that bit 5 (0x20) must be zero and bit 0 (0x01) 
        // must be 1, but real-life objects from MS tools don't obey this.

        let bytes = if is32 { 4 } else { 2 };

        let start_address = if !has_start { None } else  {
            let fix_data = self.next_uint(1)? as u8;
            let f_thread = (fix_data & 0x80) != 0;
            let f_method: FrameMethod = ((fix_data >> 4) & 7).try_into()?;
            let t_thread = (fix_data & 0x08) != 0;
            let p_displ = (fix_data & 0x04) != 0;

            let frame_datum = if f_thread || !f_method.has_datum() { None } else { self.next_opt_index()? };
            let target_datum = if t_thread { None } else { self.next_opt_index()? };
            let target_disp = if !p_displ { Some(self.next_uint(bytes)? as u32) } else { None };   
            Some(StartAddress{ fix_data, frame_datum, target_datum, target_disp })
        };

        Ok(Record::MODEND{ main, start_address })
    }

    fn lnames(&mut self) -> Result<Record, ObjError> {
        let mut names = Vec::new();

        while self.ptr < self.endrec() {
            names.push(self.next_str()?);
        }
    
        Ok(Record::LNAMES{ names })
    }

    fn segdef(&mut self, is32: bool) -> Result<Record, ObjError> {
        let mut segs = Vec::new();

        let bytes = if is32 { 4 } else { 2 };

        while self.ptr < self.endrec() {
            let acbp = self.next_uint(1)? as u8;

            let align = (acbp >> 5).try_into()?; 
            let combine = ((acbp >> 2) & 7).try_into()?;
            let big = (acbp & 2) != 0;
            let use32 = (acbp & 1) != 0;

            let abs = if align == Align::Absolute {
                let frame = self.next_uint(2)? as u16;
                let offset = self.next_uint(1)? as u8;

                Some(AbsoluteSeg { frame, offset })
            } else {
                None
            };

            let mut length = self.next_uint(bytes)?;
            
            if big {
                if length != 0 {
                    return Err(self.err("length not zero when BIG bit it set"));
                }
                length = 1 << if is32 { 32 } else { 16 };
            }

            let class = self.next_opt_index()?;
            let name = self.next_opt_index()?;
            let overlay = self.next_opt_index()?;
            
            segs.push(Segdef{
                align,
                combine,
                use32,
                abs,
                length: length as u64,
                class,
                name,
                overlay
            });
        }

        Ok(Record::SEGDEF{ segs })
    }

    fn grpdef(&mut self) -> Result<Record, ObjError> {
        let name = self.next_index()?;
        let mut segs = Vec::new();

        while self.ptr < self.endrec() {
            let typ = self.next_uint(1)?;
            let index = self.next_index()?;
            
            if typ != 0xff {
                return Err(self.err("grpdef segment with type other than FF"));
            }

            segs.push(index);
        }

        Ok(Record::GRPDEF{ name, segs })
    }

    fn extdef(&mut self) -> Result<Record, ObjError> {
        let mut externs = Vec::new();

        while self.ptr < self.endrec() {
            let name = self.next_str()?;
            let typeidx = self.next_index()?;

            externs.push(Extern{ name, typeidx });
        }

        Ok(Record::EXTDEF{ externs })
    }

    fn pubdef(&mut self, is32: bool) -> Result<Record, ObjError> {
        let group = self.next_opt_index()?;
        let seg = self.next_opt_index()?;

        let frame = if group.is_none() && seg.is_none() {
            Some(self.next_uint(2)? as u16)
        } else {
            None
        };

        let mut publics = Vec::new();

        let bytes = if is32 { 4 } else { 2 };

        while self.ptr < self.endrec() {
            let name = self.next_str()?;
            let offset = self.next_uint(bytes)? as u32;
            let typeidx = self.next_index()?;

            publics.push(Public{ name, offset, typeidx });
        }

        Ok(Record::PUBDEF{ group, seg, frame, publics })
    }

    fn ledata(&mut self, is32: bool) -> Result<Record, ObjError> {
        let seg = self.next_index()?;
        let bytes = if is32 { 4 } else { 2 };
        let offset = self.next_uint(bytes)? as u32;
        let data = &self.obj[self.ptr..self.endrec()];

        Ok(Record::LEDATA{ seg, offset, data: data.to_vec() })
    }

    fn bakpat(&mut self, is32: bool) -> Result<Record, ObjError> {
        let seg = self.next_index()?;
        let location = (self.next_uint(1)? as u8).try_into()?;

        let mut fixups = Vec::new();

        let bytes = if is32 { 4 } else { 2 };
        while self.ptr < self.endrec() {
            let offset = self.next_uint(bytes)? as u32;
            let value = self.next_uint(bytes)? as u32;
            fixups.push(BakpatFixup{ offset, value });
        }

        Ok(Record::BAKPAT{ seg, location, fixups })
    }

    fn coment_translator(&mut self, header: ComentHeader) -> Result<Record, ObjError> {
        let text = self.rest_str()?;
        Ok(Record::COMENT{
            header,
            coment: Coment::Translator{ text }
        })
    }

    fn coment_new_omf(&mut self, header: ComentHeader) -> Result<Record, ObjError> {
        let text = self.rest_str()?;
        Ok(Record::COMENT{
            header,
            coment: Coment::NewOMF{ text }
        })
    }

    fn coment_memory_model(&mut self, header: ComentHeader) -> Result<Record, ObjError> {
        let text = self.rest_str()?;
        Ok(Record::COMENT{
            header,
            coment: Coment::MemoryModel{ text }
        })
    }

    fn coment_default_library(&mut self, header: ComentHeader) -> Result<Record, ObjError> {
        let name = self.rest_str()?;
        Ok(Record::COMENT{
            header,
            coment: Coment::DefaultLibrary{ name }
        })
    }

    fn coment(&mut self) -> Result<Record, ObjError> {
        let comtype = self.next_uint(1)? as u8;
        let comclass = self.next_uint(1)? as u8;

        let header = ComentHeader{ comtype, comclass };

        match comclass {
            0x00 => self.coment_translator(header),
            0x9d => self.coment_memory_model(header),
            0x9f => self.coment_default_library(header),
            0xa1 => self.coment_new_omf(header),
            _ => Ok(Record::COMENT{ header, coment: Coment::Unknown }), 
        }
    }

    fn record(&mut self, rectype: u8) -> Result<Record, ObjError> {
        match rectype {
            0x80 => Ok(Record::THEADR{ name: self.next_str()? }),
            0x88 => self.coment(),
            0x8a => self.modend(false),
            0x8b => self.modend(true),
            0x8c => self.extdef(),
            0x90 => self.pubdef(false),
            0x91 => self.pubdef(true),
            0x96 => self.lnames(),
            0x98 => self.segdef(false),
            0x99 => self.segdef(true),
            0x9a => self.grpdef(),
            0xa0 => self.ledata(false),
            0xa1 => self.ledata(true),
            0xb2 => self.bakpat(false),
            0xb3 => self.bakpat(true),
            rectype => Ok(Record::Unknown{ rectype }),
        }
    }

    pub fn next(&mut self) -> Result<Record, ObjError> {
        self.ptr = self.next;
        self.next = self.obj.len();

        if self.ptr >= self.obj.len() {
            Ok(Record::None)
        } else if self.next - self.ptr < 3  {
            Err(self.err("record header truncated"))
        } else {
            let typ = self.next_uint(1)?;
            let len = self.next_uint(2)?;
            
            if self.ptr + len > self.obj.len() {
                Err(self.err("record body truncated"))
            } else {
                self.next = self.ptr + len;
                if !Self::checksum(&self.obj[self.start..self.next]) {
                    Err(self.err("checksum failed"))
                } else {
                    self.record(typ as u8)
                }    
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    //
    // uint
    //
    #[test]
    fn test_uint_returns_short() {
        let bytes = [0x34, 0x12];
        assert_eq!(Parser::uint(&bytes), 0x1234);
    }

    #[test]
    fn test_uint_returns_long() {
        let bytes = [0x78, 0x56, 0x34, 0x12];
        assert_eq!(Parser::uint(&bytes), 0x12345678);
    }

    //
    // parser basics
    //
    #[test]
    fn test_empty_parser_returns_none() {
        let obj = vec![];
        let mut parser = Parser::new(obj);

        let p = parser.next();
        assert!(p.is_ok(), "parser returned error {:x?}", p);
        assert_eq!(p.unwrap(), Record::None);
    }

    #[test]
    fn test_truncated_header_returns_error() {
        let obj = vec![0x42, 0x00];
        let mut parser = Parser::new(obj);

        let p = parser.next();
        assert!(p.is_err());
    }

    #[test]
    fn test_undefined_rectype_returns_unknown() {
        let obj = vec![0x42, 0x00, 0x00, 0x00];
        let mut parser = Parser::new(obj);

        let p = parser.next();
        assert!(p.is_ok(), "parser returned error {:?}", p);
        assert_eq!(p.unwrap(), Record::Unknown{ rectype: 0x42 });
    }

    #[test]
    fn test_bad_checksum_fails() {
        let obj = vec![
            0x80, 0x0e, 0x00, 0x0c,  0x64, 0x6f, 0x73, 0x5c, 
            0x63, 0x72, 0x74, 0x30,  0x2e, 0x61, 0x73, 0x6d, 
            0xdd];
        let mut parser = Parser::new(obj);

        assert!(parser.next().is_err());
    }

    #[test]
    fn test_truncated_record_fails() {
        let obj = vec![
            0x80, 0x0e, 0x00, 0x0c,  0x64, 0x6f, 0x73, 0x5c, 
            0xdc];
        let mut parser = Parser::new(obj);

        assert!(parser.next().is_err());
    }

    //
    // THEADR
    //
    #[test]
    fn test_theadr_succeeds() {
        let obj = vec![
            0x80, 0x0e, 0x00, 0x0c,  0x64, 0x6f, 0x73, 0x5c, 
            0x63, 0x72, 0x74, 0x30,  0x2e, 0x61, 0x73, 0x6d, 
            0xdc];
        let mut parser = Parser::new(obj);

        match parser.next() {
            Ok(Record::THEADR{ name }) => assert_eq!(name, "dos\\crt0.asm"),
            x => assert!(false, "parser returned {:x?}", x),
        };
    }

    //
    // LNAMES
    //
    #[test]
    fn test_lnames_succeeds() {
        let obj = vec![
            0x96, 0x09, 0x00, 0x03,  0x41, 0x42, 0x43, 0x03, 
            0x44, 0x45, 0x46, 0x00];
        let mut parser = Parser::new(obj);

        match parser.next() {
            Ok(Record::LNAMES{ names }) => {
                assert_eq!(names.len(), 2);
                assert_eq!(names[0], "ABC");
                assert_eq!(names[1], "DEF");
            },
            x => assert!(false, "parser returned {:x?}", x),
        };
    }

    //
    // SEGDEF
    //
    #[test]
    fn test_segdef_relocatable_succeeds() {
        let obj = vec![
            0x98, 0x0d, 0x00,
            0b01001000, 0x34, 0x12, 0x01, 0x02, 0x03,
            0b01100011, 0x00, 0x00, 0x05, 0x06, 0x00,
            0x00];
        let mut parser = Parser::new(obj);

        match parser.next() {
            Ok(Record::SEGDEF{ segs }) => {
                assert_eq!(segs.len(), 2);
                assert_eq!(segs[0], Segdef{
                    align: Align::Word,
                    combine: Combine::Public,
                    use32: false,
                    abs: None,
                    length: 0x1234,
                    class: Some(1),
                    name: Some(2),
                    overlay: Some(3),                
                });
                assert_eq!(segs[1], Segdef{
                    align: Align::Paragraph,
                    combine: Combine::Private,
                    use32: true,
                    abs: None,
                    length: 0x10000,
                    class: Some(5),
                    name: Some(6),
                    overlay: None,                
                });
            },
            x => assert!(false, "parser returned {:x?}", x),
        };
    }

    #[test]
    fn test_segdef_absolute_succeeds() {
        let obj = vec![
            0x98, 0x0a, 0x00,
            0b00011000, 0xee, 0xff, 0x73, 0x34, 0x12, 0x01, 0x02, 0x03,
            0x00];
        let mut parser = Parser::new(obj);

        match parser.next() {
            Ok(Record::SEGDEF{ segs }) => {
                assert_eq!(segs.len(), 1);
                assert_eq!(segs[0], Segdef{
                    align: Align::Absolute,
                    combine: Combine::Common,
                    use32: false,
                    abs: Some(AbsoluteSeg {
                        frame: 0xffee,
                        offset: 0x73,
                    }),
                    length: 0x1234,
                    class: Some(1),
                    name: Some(2),
                    overlay: Some(3),                
                });
            },
            x => assert!(false, "parser returned {:x?}", x),
        };
    }

    #[test]
    fn test_segdef_32_bit_succeeds() {
        let obj = vec![
            0x99, 0x1c, 0x00,
            0b10011000, 0x78, 0x56, 0x34, 0x12, 0x01, 0x02, 0x03,
            0b00010100, 0xee, 0xff, 0x73, 0x78, 0x56, 0x34, 0x12, 0x01, 0x02, 0x03,
            0b10011010, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03,
            0x00];
        let mut parser = Parser::new(obj);

        match parser.next() {
            Ok(Record::SEGDEF{ segs }) => {
                assert_eq!(segs.len(), 3);
                assert_eq!(segs[0], Segdef{
                    align: Align::Page,
                    combine: Combine::Common,
                    use32: false,
                    abs: None,
                    length: 0x12345678,
                    class: Some(1),
                    name: Some(2),
                    overlay: Some(3),                
                });
                assert_eq!(segs[1], Segdef{
                    align: Align::Absolute,
                    combine: Combine::Stack,
                    use32: false,
                    abs: Some(AbsoluteSeg {
                        frame: 0xffee,
                        offset: 0x73,
                    }),
                    length: 0x12345678,
                    class: Some(1),
                    name: Some(2),
                    overlay: Some(3),                
                });
                assert_eq!(segs[2], Segdef{
                    align: Align::Page,
                    combine: Combine::Common,
                    use32: false,
                    abs: None,
                    length: 0x1_0000_0000,
                    class: Some(1),
                    name: Some(2),
                    overlay: Some(3),                
                });
            },
            x => assert!(false, "parser returned {:x?}", x),
        };
    }

    //
    // GRPDEF
    //
    #[test]
    fn test_grpdef_succeeds() {
        let obj = vec![
            0x9a, 0x07, 0x00,
            0x81, 0x23, 0xff, 0x01, 0xff, 0x02,
            0x00];

        let mut parser = Parser::new(obj);

        match parser.next() {
            Ok(Record::GRPDEF{ name, segs }) => {
                assert_eq!(name, 0x0123);
                assert_eq!(segs, vec![1, 2]);
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    //
    // EXTDEF
    //
    #[test]
    fn test_extdef_succeeds() {
        let obj = vec![
            0x8c, 0x0b, 0x00,
            0x03, 0x41, 0x42, 0x43, 0x01,
            0x03, 0x44, 0x45, 0x46, 0x02,
            0x00];

        let mut parser = Parser::new(obj);

        match parser.next() {
            Ok(Record::EXTDEF{ externs }) => {
                assert_eq!(
                    externs,
                    vec![
                        Extern{ name: "ABC".to_string(), typeidx: 1},
                        Extern{ name: "DEF".to_string(), typeidx: 2},
                    ]
                );
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    //
    // PUBDEF
    //
    #[test]
    fn test_pubdef_succeeds() {
        let obj = vec![
            0x90, 0x0c, 0x00,
            0x00, 0x01, 
            0x05, 0x47, 0x41, 0x4d, 0x4d, 0x41,
            0x02, 0x00, 0x00,
            0xf9];

        let mut parser = Parser::new(obj);

        match parser.next() {
            Ok(Record::PUBDEF{ group, seg, frame, publics }) => {
                assert_eq!(group, None);
                assert_eq!(seg, Some(1));
                assert_eq!(frame, None);
                assert_eq!(
                    publics,
                    vec![
                        Public{ name: "GAMMA".to_string(), offset: 2, typeidx: 0},
                    ]
                );
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    fn test_pubdef_with_frame_succeeds() {
        let obj = vec![
            0x90, 0x0e, 0x00,
            0x00, 0x00, 0x00, 0xf0, 
            0x05, 0x47, 0x41, 0x4d, 0x4d, 0x41,
            0x34, 0x02, 0x00,
            0x00];

        let mut parser = Parser::new(obj);

        match parser.next() {
            Ok(Record::PUBDEF{ group, seg, frame, publics }) => {
                assert_eq!(group, None);
                assert_eq!(seg, None);
                assert_eq!(frame, Some(0xf000));
                assert_eq!(
                    publics,
                    vec![
                        Public{ name: "GAMMA".to_string(), offset: 0x234, typeidx: 0},
                    ]
                );
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    fn test_pubdef_with_32_bit_offset_succeeds() {
        let obj = vec![
            0x91, 0x0e, 0x00,
            0x02, 0x00, 
            0x05, 0x47, 0x41, 0x4d, 0x4d, 0x41,
            0x78, 0x56, 0x34, 0x02, 0x00,
            0x00];

        let mut parser = Parser::new(obj);

        match parser.next() {
            Ok(Record::PUBDEF{ group, seg, frame, publics }) => {
                assert_eq!(group, Some(2));
                assert_eq!(seg, None);
                assert_eq!(frame, None);
                assert_eq!(
                    publics,
                    vec![
                        Public{ name: "GAMMA".to_string(), offset: 0x2345678, typeidx: 0},
                    ]
                );
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    //
    // MODEND
    //
    #[test]
    fn test_modend_succeeds() {
        let obj = vec![
            0x8a, 0x02, 0x00, 0x01, 0x73];

        let mut parser = Parser::new(obj);

        match parser.next() {
            Ok(Record::MODEND{ main, start_address }) => {
                assert_eq!(main, false);
                assert_eq!(start_address, None);
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    fn test_modend_with_main_succeeds() {
        let obj = vec![
            0x8a, 0x02, 0x00, 0x81, 0x00];

        let mut parser = Parser::new(obj);

        match parser.next() {
            Ok(Record::MODEND{ main, start_address }) => {
                assert_eq!(main, true);
                assert_eq!(start_address, None);
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    fn test_modend_with_start_addr_succeeds() {
        let obj = vec![
            0x8a, 0x07, 0x00, 
            0xc1, 0x00, 0x01, 0x02, 0x34, 0x12, 0x00
        ];

        let mut parser = Parser::new(obj);
        match parser.next() {
            Ok(Record::MODEND{ main, start_address }) => {
                assert_eq!(main, true);
                match start_address {
                    None => assert!(false, "modend missing start address"),
                    Some(sa) => {
                        assert_eq!(sa.fix_data, 0);
                        assert_eq!(sa.frame_datum, Some(1));
                        assert_eq!(sa.target_datum, Some(2));
                        assert_eq!(sa.target_disp, Some(0x1234));
                    },
                }
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    fn test_modend_32_bits_with_start_addr_succeeds() {
        let obj = vec![
            0x8b, 0x09, 0x00, 
            0xc1, 0x00, 0x01, 0x02, 0x78, 0x56, 0x34, 0x12, 0x00
        ];

        let mut parser = Parser::new(obj);
        match parser.next() {
            Ok(Record::MODEND{ main, start_address }) => {
                assert_eq!(main, true);
                match start_address {
                    None => assert!(false, "modend missing start address"),
                    Some(sa) => {
                        assert_eq!(sa.fix_data, 0);
                        assert_eq!(sa.frame_datum, Some(1));
                        assert_eq!(sa.target_datum, Some(2));
                        assert_eq!(sa.target_disp, Some(0x12345678));
                    },
                }
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    //
    // COMENT
    //
    #[test]
    pub fn test_coment_translator_succeeds() {
        let obj = vec![
            0x88, 0x09, 0x00,
            0x00, 0x00,
            0x41, 0x42, 0x43, 0x44, 0x45, 0x46,
            0x00];

        let mut parser = Parser::new(obj);
        match parser.next() {
            Ok(Record::COMENT{ header, coment }) => {
                assert!(!header.nopurge());
                assert!(!header.nolist());

                match coment {
                    Coment::Translator{ text } => assert_eq!(text, "ABCDEF"),
                    x => assert!(false, "coment parsed was {:?}", x),
                }
            },
            x => assert!(false, "parser returned {:x?}", x),

        }
    }

    #[test]
    pub fn test_coment_new_omf_succeeds() {
        let obj = vec![
            0x88, 0x06, 0x00,
            0xc0, 0xa1,
            0x6e, 0x43, 0x56,
            0x00];

        let mut parser = Parser::new(obj);
        match parser.next() {
            Ok(Record::COMENT{ header, coment }) => {
                assert!(header.nopurge());
                assert!(header.nolist());

                match coment {
                    Coment::NewOMF{ text } => assert_eq!(text, "nCV"),
                    x => assert!(false, "coment parsed was {:?}", x),
                }
            },
            x => assert!(false, "parser returned {:x?}", x),

        }
    }

    #[test]
    pub fn test_coment_memory_model_succeeds() {
        let obj = vec![
            0x88, 0x05, 0x00,
            0x80, 0x9d,
            0x30, 0x6c,
            0x00];

        let mut parser = Parser::new(obj);
        match parser.next() {
            Ok(Record::COMENT{ header, coment }) => {
                assert!(header.nopurge());
                assert!(!header.nolist());

                match coment {
                    Coment::MemoryModel{ text } => assert_eq!(text, "0l"),
                    x => assert!(false, "coment parsed was {:?}", x),
                }
            },
            x => assert!(false, "parser returned {:x?}", x),

        }
    }

    #[test]
    pub fn test_coment_default_library_succeeds() {
        let obj = vec![
            0x88, 0x06, 0x00,
            0x40, 0x9f,
            0x41, 0x43, 0x45,
            0x00];

        let mut parser = Parser::new(obj);
        match parser.next() {
            Ok(Record::COMENT{ header, coment }) => {
                assert!(!header.nopurge());
                assert!(header.nolist());

                match coment {
                    Coment::DefaultLibrary{ name } => assert_eq!(name, "ACE"),
                    x => assert!(false, "coment parsed was {:?}", x),
                }
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    //
    // LEDATA
    //
    #[test]
    fn test_ledata_succeeds() {
        let obj = vec![
            0xa0, 0x09, 0x00, 
            0x01, 
            0x34, 0x12, 
            0x02, 0x78, 0x56, 0x34, 0x12, 
            0x00
        ];

        let mut parser = Parser::new(obj);
        match parser.next() {
            Ok(Record::LEDATA{ seg, offset, data }) => {
                assert_eq!(seg, 1);
                assert_eq!(offset, 0x1234);
                assert_eq!(data, vec![0x02, 0x78, 0x56, 0x34, 0x12]);
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    fn test_ledata32_succeeds() {
        let obj = vec![
            0xa1, 0x0b, 0x00, 
            0x01, 
            0x78, 0x56, 0x34, 0x12, 
            0x02, 0x78, 0x56, 0x34, 0x12, 
            0x00
        ];

        let mut parser = Parser::new(obj);
        match parser.next() {
            Ok(Record::LEDATA{ seg, offset, data }) => {
                assert_eq!(seg, 1);
                assert_eq!(offset, 0x12345678);
                assert_eq!(data, vec![0x02, 0x78, 0x56, 0x34, 0x12]);
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    //
    // BAKPAT
    //
    #[test]
    fn test_bakpat_succeeds() {
        let obj = vec![
            0xb2, 0x0b, 0x00, 
            0x01,
            0x01,
            0x02, 0x00, 0x34, 0x12,
            0x05, 0x01, 0x78, 0x56,
            0x00
        ];

        let mut parser = Parser::new(obj);
        match parser.next() {
            Ok(Record::BAKPAT{ seg, location, fixups }) => {
                assert_eq!(seg, 1);
                assert_eq!(location, BakpatLocation::Word);
                assert_eq!(fixups, vec![
                    BakpatFixup{ offset: 0x0002, value: 0x1234 },
                    BakpatFixup{ offset: 0x0105, value: 0x5678 },
                ]);
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    fn test_bakpat32_succeeds() {
        let obj = vec![
            0xb3, 0x0b, 0x00, 
            0x01,
            0x02,
            0x02, 0x00, 0x01, 0x00, 0x34, 0x12, 0x55, 0xaa,
            0x00
        ];

        let mut parser = Parser::new(obj);
        match parser.next() {
            Ok(Record::BAKPAT{ seg, location, fixups }) => {
                assert_eq!(seg, 1);
                assert_eq!(location, BakpatLocation::Dword);
                assert_eq!(fixups, vec![
                    BakpatFixup{ offset: 0x00010002, value: 0xaa551234 },
                ]);
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }
}

