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
pub enum FixupLocation {
    Byte,
    Word,
    Selector,
    LongPointer,
    LoaderWord,
    HighOrderByte,
    Offset32,
    LoaderOffset32,
    Pointer48,    
}

impl TryFrom<u8> for FixupLocation {
    type Error = ObjError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(FixupLocation::Byte),
            1 => Ok(FixupLocation::Word),
            2 => Ok(FixupLocation::Selector),
            3 => Ok(FixupLocation::LongPointer),
            5 => Ok(FixupLocation::LoaderWord),
            9 => Ok(FixupLocation::Offset32),
            11 => Ok(FixupLocation::Pointer48),
            13 => Ok(FixupLocation::LoaderOffset32),

            val => Err(ObjError::new(&format!("invalid fixup location ${:02x}", val))),
        }
    }
}

// NB most enum cases have the data directly embedded, but fixup has enough
// fields that it's unwieldy
//
#[derive(Debug)]
#[derive(PartialEq)]
pub struct Fixup {
    pub is_seg_relative: bool,
    pub location: FixupLocation,
    pub data_offset: usize,
    pub frame_thread: Option<usize>,
    pub frame_method: Option<FrameMethod>,
    pub frame_datum: Option<usize>,
    pub target_thread: Option<usize>,
    pub target_method: Option<TargetMethod>,
    pub target_datum: Option<usize>,
    pub target_displacement: u32,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum FixupSubrecord {
    TargetThread{ method: TargetMethod, thread: usize, index: usize },
    FrameThread{ method: FrameMethod, thread: usize, index: Option<usize> },
    Fixup{ fixup: Fixup }, 
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
pub struct Comdef {
    pub name: String,
    pub length: usize,
    pub datatype: u8,
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
pub struct WeakExtern {
    pub weak: usize,
    pub default: usize,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum Coment {
    Unknown,
    Translator{ text: String },
    MemoryModel{ text: String },
    DosSeg,
    DefaultLibrary{ name: String },
    LinkPassSeparator,
    NewOMF{ text: String },
    Libmod{ name: String },
    WeakExtern{ externs: Vec<WeakExtern> },
    User{ text: String },
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
pub struct Alias {
    pub alias: String,
    pub substitute: String,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct CExtern {
    pub name: usize,
    pub typeindex: usize,
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ComdatSelection {
    NoMatch,
    PickAny,
    SameSize,
    ExactMatch,
}

impl TryFrom<u8> for ComdatSelection {
    type Error = ObjError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val & 0xf0 {
            0x00 => Ok(ComdatSelection::NoMatch),
            0x10 => Ok(ComdatSelection::PickAny),
            0x20 => Ok(ComdatSelection::SameSize),
            0x30 => Ok(ComdatSelection::ExactMatch),

            val => Err(ObjError::new(&format!("invalid comdat selection ${:02x}", val))),
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ComdatAllocation {
    Explicit,
    FarCode,
    FarData,
    Code32,
    Data32
}

impl TryFrom<u8> for ComdatAllocation {
    type Error = ObjError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val & 0x0f {
            0x00 => Ok(ComdatAllocation::Explicit),
            0x01 => Ok(ComdatAllocation::FarCode),
            0x02 => Ok(ComdatAllocation::FarData),
            0x03 => Ok(ComdatAllocation::Code32),
            0x04 => Ok(ComdatAllocation::Data32),
            
            val => Err(ObjError::new(&format!("invalid comdat allocation ${:02x}", val))),
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ComdatAlign {
    Segdef,
    Byte,
    Word,
    Paragraph,
    Page,
    Dword
}

impl TryFrom<u8> for ComdatAlign {
    type Error = ObjError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val & 0x0f {
            0x00 => Ok(ComdatAlign::Segdef),
            0x01 => Ok(ComdatAlign::Byte),
            0x02 => Ok(ComdatAlign::Word),
            0x03 => Ok(ComdatAlign::Paragraph),
            0x04 => Ok(ComdatAlign::Page),
            0x05 => Ok(ComdatAlign::Dword),
            
            val => Err(ObjError::new(&format!("invalid comdat align ${:02x}", val))),
        }
    }
}


#[derive(Debug)]
#[derive(PartialEq)]
pub struct Comdat {
    pub flags: u8,
    pub selection: ComdatSelection,
    pub allocation: ComdatAllocation,
    pub align: ComdatAlign,
    pub offset: u32,
    pub typeindex: usize,
    pub base_group: Option<usize>,
    pub base_seg: Option<usize>,
    pub base_frame: Option<u16>,
    pub name: usize,
    pub data: Vec<u8>,
}

impl Comdat {
    pub fn continuation(&self) -> bool {
        (self.flags & 0x01) != 0
    }

    pub fn iterated_data(&self) -> bool {
        (self.flags & 0x02) != 0
    }

    pub fn local(&self) -> bool {
        (self.flags & 0x04) != 0
    }

    pub fn codeseg(&self) -> bool {
        (self.flags & 0x08) != 0
    }
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
    FIXUPP{ fixups: Vec<FixupSubrecord >},
    COMDEF { commons: Vec<Comdef> },
    CEXTDEF { externs: Vec<CExtern> },
    LEXTDEF{ externs: Vec<Extern> },
    LPUBDEF{ group: Option<usize>, seg: Option<usize>, frame: Option<u16>, publics: Vec<Public> },
    ALIAS { aliases: Vec<Alias> },
    COMDAT { comdat: Comdat },
}

pub struct Parser<'a> {
    obj: &'a [u8],
    start: usize,
    ptr: usize,
    next: usize,
}

impl<'a> Parser<'a> {
    pub fn new(obj: &'a [u8]) -> Parser<'a> {
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

    fn make_externs(&mut self, rec: &dyn Fn(Vec<Extern>) -> Record) -> Result<Record, ObjError> {
        let mut externs = Vec::new();

        while self.ptr < self.endrec() {
            let name = self.next_str()?;
            let typeidx = self.next_index()?;

            externs.push(Extern{ name, typeidx });
        }

        Ok(rec(externs))
    }

    fn extdef(&mut self) -> Result<Record, ObjError> {
        self.make_externs(&|externs| Record::EXTDEF{ externs })
    }

    fn lextdef(&mut self) -> Result<Record, ObjError> {
        self.make_externs(&|externs| Record::LEXTDEF{ externs })
    }

    fn alias(&mut self) -> Result<Record, ObjError> {
        let mut aliases = Vec::new();

        while self.ptr < self.endrec() {
            let alias = self.next_str()?;
            let substitute = self.next_str()?;

            aliases.push(Alias{ alias, substitute });
        }

        Ok(Record::ALIAS{ aliases })
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

    fn lpubdef(&mut self, is32: bool) -> Result<Record, ObjError> {
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

        Ok(Record::LPUBDEF{ group, seg, frame, publics })
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

    fn fixupp(&mut self, is32: bool) -> Result<Record, ObjError> {
        let mut fixups = Vec::new();

        while self.ptr < self.endrec() {
            let lead = self.next_uint(1)? as u8;

            if (lead & 0x80) == 0x00 {
                // thread subrecord
                let thread = (lead & 3) as usize;
                if (lead & 0x40) == 0x00 {
                    // target thread
                    // NB from spec:
                    //   "For TARGET threads, only the lower two bits of the
                    //    field are used; the high-order bit of the method is 
                    //    derived from the P bit in the Fix Data field of FIXUP 
                    //    subrecords that refer to this thread."
                    //
                    let method: TargetMethod = ((lead >> 2) & 3).try_into()?;
                    let index = self.next_index()?;
                    fixups.push(FixupSubrecord::TargetThread{ method, thread, index })
                } else {
                    // frame thread
                    let method: FrameMethod = ((lead >> 2) & 7).try_into()?;
                    let index = if method.has_datum() {
                        Some(self.next_index()?)
                    } else {
                        None
                    };

                    fixups.push(FixupSubrecord::FrameThread{ method, thread, index })
                }
            } else {
                //
                // TODO if not seg_relative is entire fixdata section missing?
                //
                let is_seg_relative = (lead & 0x40) != 0;
                let location: FixupLocation = ((lead >> 2) & 0x0f).try_into()?;
                let low = self.next_uint(1)?;
                let data_offset = (((lead as usize) & 3) << 8) | low;
                let fixdata = self.next_uint(1)?;

                let frame_uses_thread = (fixdata & 0x80) != 0;
                let target_uses_thread = (fixdata & 0x08) != 0;

                let frame_thread = if frame_uses_thread { Some((fixdata >> 4) & 3) } else { None };
                let target_thread = if target_uses_thread { Some(fixdata & 3) } else { None };

                let frame_method: Option<FrameMethod> = if frame_uses_thread {
                    None
                } else {
                    Some((((fixdata >> 4) & 7) as u8).try_into()?)
                };

                let target_method: Option<TargetMethod> = if target_uses_thread {
                    None
                } else {
                    Some(((fixdata & 7) as u8).try_into()?)
                };

                let frame_datum = match &frame_method {
                    Some(method) => if method.has_datum() {
                        Some(self.next_index()?)
                    } else {
                        None
                    },
                    None => None
                };

                let target_datum = if target_method.is_none() {
                    None
                } else {
                    Some(self.next_index()?)
                };

                let target_displacement = if (fixdata & 0x04) != 0 {
                    0
                } else {
                    let bytes = if is32 { 4 } else { 2 };
                    self.next_uint(bytes)? as u32
                };

                // fixup subrecord
                let fixup = Fixup{
                    is_seg_relative,
                    location,
                    data_offset,
                    frame_thread,
                    frame_method,
                    frame_datum,
                    target_thread,
                    target_method,
                    target_datum,
                    target_displacement,
                };

                fixups.push(FixupSubrecord::Fixup{ fixup });
            }
        }

        Ok(Record::FIXUPP{ fixups })
    }

    fn comlength(&mut self) -> Result<usize, ObjError> {
        let byte = self.next_uint(1)?;
        if byte <= 0x80 {
            Ok(byte)
        } else {
            match byte {
                0x81 => Ok(self.next_uint(2)?),
                0x82 => Ok(self.next_uint(3)?),
                0x83 => Ok(self.next_uint(4)?),
                x => return Err(self.err(&format!("invalid encoded length lead byte {:02x}", x))),
            }
        }
    }

    fn comdef(&mut self) -> Result<Record, ObjError> {
        let mut commons = Vec::new();

        while self.ptr < self.endrec() {
            let name = self.next_str()?;
            let typeidx = self.next_index()?;
            let datatype = self.next_uint(1)? as u8;
            let mut length = self.comlength()?;

            if datatype == 0x61 {
                // far length which is the product of length * element size
                length *= self.comlength()?;
            }
            
            commons.push(Comdef{
                name,
                length,
                datatype,
                typeidx,
            });
        }
        
        Ok(Record::COMDEF{ commons })
    }

    fn cextdef(&mut self) -> Result<Record, ObjError> {
        let mut externs = Vec::new();

        while self.ptr < self.endrec() {
            let name = self.next_index()?;
            let typeindex = self.next_index()?;

            externs.push(CExtern{ name, typeindex });
        }

        Ok(Record::CEXTDEF{ externs })
    }

    fn comdat(&mut self, is32: bool) -> Result<Record, ObjError> {
        let flags = self.next_uint(1)? as u8;
        let attributes = self.next_uint(1)? as u8;
        let align = self.next_uint(1)? as u8;

        let bytes = if is32 { 4 } else { 2 };
        let offset = self.next_uint(bytes)? as u32;
        let typeindex = self.next_index()?;
        let base_group = self.next_opt_index()?;
        let base_seg = self.next_opt_index()?;

        let base_frame = if base_group.is_none() && base_seg.is_none() {
            Some(self.next_uint(2)? as u16)
        } else {
            None
        };

        let name = self.next_index()?;

        let mut data = Vec::new();

        while self.ptr < self.endrec() {
            data.push(self.next_uint(1)? as u8);
        }

        Ok(Record::COMDAT{
            comdat: Comdat {
                flags,
                selection: attributes.try_into()?,
                allocation: attributes.try_into()?,
                align: align.try_into()?,
                offset,
                typeindex,
                base_group,
                base_seg,
                base_frame,
                name,
                data,    
            }
        })
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

    fn coment_libmod(&mut self, header: ComentHeader) -> Result<Record, ObjError> {
        // Unlike most other coment strings, libmod is a counted string
        //
        let name = self.next_str()?;
        Ok(Record::COMENT{
            header,
            coment: Coment::Libmod{ name }
        })
    }

    fn coment_weak_extern(&mut self, header: ComentHeader) -> Result<Record, ObjError> {
        let mut externs = Vec::new();

        while self.ptr < self.endrec() {
            let weak = self.next_index()?;
            let default = self.next_index()?;

            externs.push(WeakExtern{ weak, default });
        }
        
        
        Ok(Record::COMENT{ 
            header,
            coment: Coment::WeakExtern{ externs }
        })
    }

    fn coment_user(&mut self, header: ComentHeader) -> Result<Record, ObjError> {
        let text = self.rest_str()?;
        Ok(Record::COMENT{
            header,
            coment: Coment::User{ text }
        })
    }

    fn coment(&mut self) -> Result<Record, ObjError> {
        let comtype = self.next_uint(1)? as u8;
        let comclass = self.next_uint(1)? as u8;

        let header = ComentHeader{ comtype, comclass };

        match comclass {
            0x00 => self.coment_translator(header),
            0x9d => self.coment_memory_model(header),
            0x9e => Ok(Record::COMENT{ header, coment: Coment::DosSeg }),
            0x9f => self.coment_default_library(header),
            0xa1 => self.coment_new_omf(header),
            0xa2 => Ok(Record::COMENT{ header, coment: Coment::LinkPassSeparator }),
            0xa3 => self.coment_libmod(header),
            0xa8 => self.coment_weak_extern(header),
            0xdf => self.coment_user(header),
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
            0x9c => self.fixupp(false),
            0x9d => self.fixupp(true),
            0xa0 => self.ledata(false),
            0xa1 => self.ledata(true),
            0xb0 => self.comdef(),
            0xb2 => self.bakpat(false),
            0xb3 => self.bakpat(true),
            0xb4 => self.lextdef(),
            0xb5 => self.lextdef(), // NB defined per spec w/ no semantic difference from b4
            0xb6 => self.lpubdef(false),
            0xb7 => self.lpubdef(true),
            0xbc => self.cextdef(),
            0xc2 => self.comdat(false),
            0xc3 => self.comdat(true),
            0xc6 => self.alias(),
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
        let mut parser = Parser::new(&obj);

        let p = parser.next();
        assert!(p.is_ok(), "parser returned error {:x?}", p);
        assert_eq!(p.unwrap(), Record::None);
    }

    #[test]
    fn test_truncated_header_returns_error() {
        let obj = vec![0x42, 0x00];
        let mut parser = Parser::new(&obj);

        let p = parser.next();
        assert!(p.is_err());
    }

    #[test]
    fn test_undefined_rectype_returns_unknown() {
        let obj = vec![0x42, 0x00, 0x00, 0x00];
        let mut parser = Parser::new(&obj);

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
        let mut parser = Parser::new(&obj);

        assert!(parser.next().is_err());
    }

    #[test]
    fn test_truncated_record_fails() {
        let obj = vec![
            0x80, 0x0e, 0x00, 0x0c,  0x64, 0x6f, 0x73, 0x5c, 
            0xdc];
        let mut parser = Parser::new(&obj);

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
        let mut parser = Parser::new(&obj);

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
        let mut parser = Parser::new(&obj);

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
        let mut parser = Parser::new(&obj);

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
        let mut parser = Parser::new(&obj);

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
        let mut parser = Parser::new(&obj);

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

        let mut parser = Parser::new(&obj);

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

        let mut parser = Parser::new(&obj);

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

        let mut parser = Parser::new(&obj);

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

        let mut parser = Parser::new(&obj);

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

        let mut parser = Parser::new(&obj);

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
    // LPUBDEF
    //
    #[test]
    fn test_lpubdef_succeeds() {
        let obj = vec![
            0xb6, 0x0c, 0x00,
            0x00, 0x01, 
            0x05, 0x47, 0x41, 0x4d, 0x4d, 0x41,
            0x02, 0x00, 0x00,
            0x00];

        let mut parser = Parser::new(&obj);

        match parser.next() {
            Ok(Record::LPUBDEF{ group, seg, frame, publics }) => {
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
    fn test_lpubdef_with_frame_succeeds() {
        let obj = vec![
            0xb6, 0x0e, 0x00,
            0x00, 0x00, 0x00, 0xf0, 
            0x05, 0x47, 0x41, 0x4d, 0x4d, 0x41,
            0x34, 0x02, 0x00,
            0x00];

        let mut parser = Parser::new(&obj);

        match parser.next() {
            Ok(Record::LPUBDEF{ group, seg, frame, publics }) => {
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
    fn test_lpubdef_with_32_bit_offset_succeeds() {
        let obj = vec![
            0xb7, 0x0e, 0x00,
            0x02, 0x00, 
            0x05, 0x47, 0x41, 0x4d, 0x4d, 0x41,
            0x78, 0x56, 0x34, 0x02, 0x00,
            0x00];

        let mut parser = Parser::new(&obj);

        match parser.next() {
            Ok(Record::LPUBDEF{ group, seg, frame, publics }) => {
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

        let mut parser = Parser::new(&obj);

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

        let mut parser = Parser::new(&obj);

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

        let mut parser = Parser::new(&obj);
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

        let mut parser = Parser::new(&obj);
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

        let mut parser = Parser::new(&obj);
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

        let mut parser = Parser::new(&obj);
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
    pub fn test_coment_link_pass_sep_succeeds() {
        let obj = vec![
            0x88, 0x03, 0x00,
            0xc0, 0xa2,
            0x00];

        let mut parser = Parser::new(&obj);
        match parser.next() {
            Ok(Record::COMENT{ header, coment }) => {
                assert!(header.nopurge());
                assert!(header.nolist());

                assert_eq!(coment, Coment::LinkPassSeparator);
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

        let mut parser = Parser::new(&obj);
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
    pub fn test_coment_dosseg_succeeds() {
        let obj = vec![
            0x88, 0x03, 0x00,
            0x80, 0x9e,
            0x00];

        let mut parser = Parser::new(&obj);
        match parser.next() {
            Ok(Record::COMENT{ header, coment }) => {
                assert!(header.nopurge());
                assert!(!header.nolist());

                match coment {
                    Coment::DosSeg => (),
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

        let mut parser = Parser::new(&obj);
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

    #[test]
    pub fn test_coment_libmod_succeeds() {
        let obj = vec![
            0x88, 0x09, 0x00,
            0x00, 0xa3, 
            0x05, 0x41, 0x42, 0x43, 0x44, 0x45,
            0x00];

        let mut parser = Parser::new(&obj);
        match parser.next() {
            Ok(Record::COMENT{ header: _, coment }) => {
                match coment {
                    Coment::Libmod{ name } => assert_eq!(name, "ABCDE"),
                    x => assert!(false, "coment parsed was {:?}", x),
                }
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }


    #[test]
    pub fn test_coment_weak_extern_succeeds() {
        let obj = vec![
            0x88, 0x08, 0x00,
            0x00, 0xa8, 
            0x01, 0x02, 
            0x03, 0x81, 0x23,
            0x00];

        let mut parser = Parser::new(&obj);
        match parser.next() {
            Ok(Record::COMENT{ header: _, coment }) => {
                match coment {
                    Coment::WeakExtern{ externs } => assert_eq!(externs, vec![
                        WeakExtern{ weak: 1, default: 2 },
                        WeakExtern{ weak: 3, default: 0x123 },
                    ]),
                    x => assert!(false, "coment parsed was {:?}", x),
                }
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    pub fn test_coment_user_succeeds() {
        let obj = vec![
            0x88, 0x08, 0x00,
            0x00, 0xdf, 
            0x41, 0x42, 0x43, 0x44, 0x45,
            0x00];

        let mut parser = Parser::new(&obj);
        match parser.next() {
            Ok(Record::COMENT{ header: _, coment }) => {
                match coment {
                    Coment::User{ text } => assert_eq!(text, "ABCDE"),
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

        let mut parser = Parser::new(&obj);
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

        let mut parser = Parser::new(&obj);
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

        let mut parser = Parser::new(&obj);
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

        let mut parser = Parser::new(&obj);
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

    //
    // FIXUPP
    //
    #[test]
    fn test_fixup_frame_thread_succeeds() {
        let obj = vec![
            0x9c, 0x03, 0x00, 
            0b010_001_01,
            0x07,
            0x00
        ];

        let mut parser = Parser::new(&obj);
        match parser.next() {
            Ok(Record::FIXUPP{ fixups }) => {
                assert_eq!(fixups, vec![
                    FixupSubrecord::FrameThread{
                        method: FrameMethod::Grpdef,
                        thread: 1,
                        index: Some(7)
                    }
                ]);
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    fn test_fixup_frame_thread_no_datum_succeeds() {
        let obj = vec![
            0x9c, 0x02, 0x00, 
            0b010_101_01,
            0x00
        ];

        let mut parser = Parser::new(&obj);
        match parser.next() {
            Ok(Record::FIXUPP{ fixups }) => {
                assert_eq!(fixups, vec![
                    FixupSubrecord::FrameThread{
                        method: FrameMethod::Target,
                        thread: 1,
                        index: None,
                    }
                ]);
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    fn test_fixup_target_thread_succeeds() {
        let obj = vec![
            0x9c, 0x03, 0x00, 
            0b000_010_10,
            0x06,
            0x00
        ];

        let mut parser = Parser::new(&obj);
        match parser.next() {
            Ok(Record::FIXUPP{ fixups }) => {
                assert_eq!(fixups, vec![
                    FixupSubrecord::TargetThread{
                        method: TargetMethod::Extdef,
                        thread: 2,
                        index: 6
                    }
                ]);
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    fn test_fixup_succeeds() {
        let obj = vec![
            0x9c, 0x08, 0x00, 
            0b1_1_0001_00, 0x67,
            0b0_001_0_000,
            0x01,
            0x02,
            0x34, 0x12,
            0x00
        ];

        let mut parser = Parser::new(&obj);
        match parser.next() {
            Ok(Record::FIXUPP{ fixups }) => {
                assert_eq!(fixups, vec![
                    FixupSubrecord::Fixup{
                        fixup: Fixup {
                            is_seg_relative: true,
                            location: FixupLocation::Word,
                            data_offset: 0x0067,
                            frame_thread: None,
                            frame_method: Some(FrameMethod::Grpdef),
                            frame_datum: Some(1),
                            target_thread: None,
                            target_method: Some(TargetMethod::Segdef),
                            target_datum: Some(2),
                            target_displacement: 0x1234,
                        }
                    }
                ]);
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    fn test_fixup_using_thread_succeeds() {
        let obj = vec![
            0x9c, 0x06, 0x00, 
            0b1_1_0001_00, 0x67,
            0b1_001_1_010,
            0x34, 0x12,
            0x00
        ];

        let mut parser = Parser::new(&obj);
        match parser.next() {
            Ok(Record::FIXUPP{ fixups }) => {
                assert_eq!(fixups, vec![
                    FixupSubrecord::Fixup{
                        fixup: Fixup {
                            is_seg_relative: true,
                            location: FixupLocation::Word,
                            data_offset: 0x0067,
                            frame_thread: Some(1),
                            frame_method: None,
                            frame_datum: None,
                            target_thread: Some(2),
                            target_method: None,
                            target_datum: None,
                            target_displacement: 0x1234,
                        }
                    }
                ]);
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }


    #[test]
    fn test_fixup_no_displacement_succeeds() {
        let obj = vec![
            0x9c, 0x04, 0x00, 
            0b1_1_0001_00, 0x67,
            0b1_001_1_110,
            0x00
        ];

        let mut parser = Parser::new(&obj);
        match parser.next() {
            Ok(Record::FIXUPP{ fixups }) => {
                assert_eq!(fixups, vec![
                    FixupSubrecord::Fixup{
                        fixup: Fixup {
                            is_seg_relative: true,
                            location: FixupLocation::Word,
                            data_offset: 0x0067,
                            frame_thread: Some(1),
                            frame_method: None,
                            frame_datum: None,
                            target_thread: Some(2),
                            target_method: None,
                            target_datum: None,
                            target_displacement: 0,
                        }
                    }
                ]);
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    fn test_32_bit_fixup_succeeds() {
        let obj = vec![
            0x9d, 0x0a, 0x00, 
            0b1_1_0001_00, 0x67,
            0b0_001_0_000,
            0x01,
            0x02,
            0x78, 0x56, 0x34, 0x12,
            0x00
        ];

        let mut parser = Parser::new(&obj);
        match parser.next() {
            Ok(Record::FIXUPP{ fixups }) => {
                assert_eq!(fixups, vec![
                    FixupSubrecord::Fixup{
                        fixup: Fixup {
                            is_seg_relative: true,
                            location: FixupLocation::Word,
                            data_offset: 0x0067,
                            frame_thread: None,
                            frame_method: Some(FrameMethod::Grpdef),
                            frame_datum: Some(1),
                            target_thread: None,
                            target_method: Some(TargetMethod::Segdef),
                            target_datum: Some(2),
                            target_displacement: 0x12345678,
                        }
                    }
                ]);
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    //
    // COMDEF
    //
    #[test]
    fn test_comdef_succeeds() {
        let obj = vec![
            0xb0, 0x20, 0x00,
            0x04, 0x5f, 0x66, 0x6f, 0x6f, 0x00, 0x62, 0x02,
            0x05, 0x5f, 0x66, 0x6f, 0x6f, 0x32, 0x00, 0x62, 0x81, 0x00, 0x80,
            0x05, 0x5f, 0x66, 0x6f, 0x6f, 0x33, 0x00, 0x61, 0x81, 0x90, 0x01, 0x01,
            0x99
        ];

        let mut parser = Parser::new(&obj);
        match parser.next() {
            Ok(Record::COMDEF{ commons }) => {
                assert_eq!(commons, vec![
                    Comdef{ 
                        name: "_foo".to_string(),
                        length: 2,
                        datatype: 0x62,
                        typeidx: 0
                    },
                    Comdef{ 
                        name: "_foo2".to_string(),
                        length: 32768,
                        datatype: 0x62,
                        typeidx: 0
                    },
                    Comdef{ 
                        name: "_foo3".to_string(),
                        length: 400,
                        datatype: 0x61,
                        typeidx: 0
                    },
                ]);
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    //
    // LEXTDEF
    //
    #[test]
    fn test_lextdef_succeeds() {
        let obj = vec![
            0xb4, 0x0b, 0x00,
            0x03, 0x41, 0x42, 0x43, 0x01,
            0x03, 0x44, 0x45, 0x46, 0x02,
            0x00];

        let mut parser = Parser::new(&obj);

        match parser.next() {
            Ok(Record::LEXTDEF{ externs }) => {
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
    // ALIAS
    //
    #[test]
    fn test_alias_succeeds() {
        let obj = vec![
            0xc6, 0x11, 0x00,
            0x03, 0x41, 0x42, 0x43,
            0x03, 0x44, 0x45, 0x46,
            0x03, 0x47, 0x48, 0x49,
            0x03, 0x4a, 0x4b, 0x4c,
            0x00];

        let mut parser = Parser::new(&obj);

        match parser.next() {
            Ok(Record::ALIAS{ aliases }) => {
                assert_eq!(
                    aliases,
                    vec![
                        Alias{ alias: "ABC".to_string(), substitute: "DEF".to_string() },
                        Alias{ alias: "GHI".to_string(), substitute: "JKL".to_string() },
                    ]
                );
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    //
    // CEXTDEF
    //
    #[test]
    fn test_cextdef_succeeds() {
        let obj = vec![
            0xbc, 0x05, 0x00,
            0x01, 0x00, 0x02, 0x03,
            0x00];

        let mut parser = Parser::new(&obj);

        match parser.next() {
            Ok(Record::CEXTDEF{ externs }) => {
                assert_eq!(
                    externs,
                    vec![
                        CExtern{ name: 1, typeindex: 0 },
                        CExtern{ name: 2, typeindex: 3 },
                    ]
                );
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    //
    // COMDAT
    //
    #[test]
    fn test_comdat_succeeds() {
        let obj = vec![
            0xc2, 0x0c, 0x00,
            0x01,           // flags 
            0x10,           // attributs
            0x00,           // align
            0x34, 0x12,     // data offset
            0x01,           // type index
            0x01,           // base group
            0x02,           // base segment
            0x03,           // name
            0x55, 0x66,     // data
            0x00];

        let mut parser = Parser::new(&obj);

        match parser.next() {
            Ok(Record::COMDAT{ comdat }) => {
                assert_eq!(
                    comdat,
                    Comdat {
                        flags: 0x01,
                        selection: ComdatSelection::PickAny,
                        allocation: ComdatAllocation::Explicit,
                        align: ComdatAlign::Segdef,
                        offset: 0x1234,
                        typeindex: 1,
                        base_group: Some(1),
                        base_seg: Some(2),
                        base_frame: None,
                        name: 3,
                        data: vec![0x55, 0x66],
                    }
                );
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    fn test_comdat_far_code_succeeds() {
        let obj = vec![
            0xc2, 0x0c, 0x00,
            0x01,           // flags 
            0x11,           // attributs
            0x00,           // align
            0x34, 0x12,     // data offset
            0x01,           // type index
            0x01,           // base group
            0x02,           // base segment
            0x03,           // name
            0x55, 0x66,     // data
            0x00];

        let mut parser = Parser::new(&obj);

        match parser.next() {
            Ok(Record::COMDAT{ comdat }) => {
                assert_eq!(
                    comdat,
                    Comdat {
                        flags: 0x01,
                        selection: ComdatSelection::PickAny,
                        allocation: ComdatAllocation::FarCode,
                        align: ComdatAlign::Segdef,
                        offset: 0x1234,
                        typeindex: 1,
                        base_group: Some(1),
                        base_seg: Some(2),
                        base_frame: None,
                        name: 3,
                        data: vec![0x55, 0x66],
                    }
                );
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    fn test_comdat_frame_succeeds() {
        let obj = vec![
            0xc2, 0x0e, 0x00,
            0x01,           // flags 
            0x10,           // attributs
            0x00,           // align
            0x34, 0x12,     // data offset
            0x01,           // type index
            0x00,           // base group
            0x00,           // base segment
            0x00, 0xf0,     // base frame
            0x03,           // name
            0x55, 0x66,     // data
            0x00];

        let mut parser = Parser::new(&obj);

        match parser.next() {
            Ok(Record::COMDAT{ comdat }) => {
                assert_eq!(
                    comdat,
                    Comdat {
                        flags: 0x01,
                        selection: ComdatSelection::PickAny,
                        allocation: ComdatAllocation::Explicit,
                        align: ComdatAlign::Segdef,
                        offset: 0x1234,
                        typeindex: 1,
                        base_group: None,
                        base_seg: None,
                        base_frame: Some(0xf000),
                        name: 3,
                        data: vec![0x55, 0x66],
                    }
                );
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

    #[test]
    fn test_comdat32_succeeds() {
        let obj = vec![
            0xc3, 0x0e, 0x00,
            0x01,           // flags 
            0x10,           // attributs
            0x00,           // align
            0x78, 0x56, 0x34, 0x12,     // data offset
            0x01,           // type index
            0x01,           // base group
            0x02,           // base segment
            0x03,           // name
            0x55, 0x66,     // data
            0x00];

        let mut parser = Parser::new(&obj);

        match parser.next() {
            Ok(Record::COMDAT{ comdat }) => {
                assert_eq!(
                    comdat,
                    Comdat {
                        flags: 0x01,
                        selection: ComdatSelection::PickAny,
                        allocation: ComdatAllocation::Explicit,
                        align: ComdatAlign::Segdef,
                        offset: 0x12345678,
                        typeindex: 1,
                        base_group: Some(1),
                        base_seg: Some(2),
                        base_frame: None,
                        name: 3,
                        data: vec![0x55, 0x66],
                    }
                );
            },
            x => assert!(false, "parser returned {:x?}", x),
        }
    }

}

