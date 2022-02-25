mod args;

use std::str;

use dt_lib::error::Error as AppError;
use dt_lib::objfile::*;
use dt_lib::libfile;

use crate::args::Args;

struct Objdump {
    lnames: Vec<String>,
    segments: Vec<Segdef>,
    groups: Vec<String>,
    externs: Vec<String>,
}

impl Objdump {
    fn new() -> Objdump {
        Objdump {
            lnames: vec!["".to_string()],
            segments: vec![Segdef::empty()],
            groups: vec!["".to_string()],
            externs: vec!["".to_string()],
        }
    }

    fn lnames(&mut self, names: &[String]) -> Result<(), AppError> {
        println!("LNAMES");
        for name in names.iter() {
            println!("{:5} {}", self.lnames.len(), name);
            self.lnames.push(name.clone());
        }

        Ok(())
    }

    fn lname(&self, index: usize) -> &str {
        if index >= self.lnames.len() {
            "invalid-lname"
        } else {
            &self.lnames[index]
        }
    }

    fn opt_lname(&self, index: Option<usize>) -> &str {
        match index {
            Some(index) => self.lname(index),
            None => "null",
        }
    }

    fn segname(&self, seg: &Segdef) -> String {
        format!("{}.{}.{}",
            self.opt_lname(seg.class),
            self.opt_lname(seg.name),
            self.opt_lname(seg.overlay)
        )
    }

    fn segdef(&mut self, segs: &[Segdef]) -> Result<(), AppError> {
        println!("SEGDEF");
        for seg in segs.iter() {
            print!("{:5} {} {:?} {:?}",
                self.segments.len(),
                self.segname(seg),
                seg.align,
                seg.combine,
            );
            
            if let Some(abs) = &seg.abs {
                print!(" Frame={:04x}:{:02x}", abs.frame, abs.offset);
            }

            if seg.use32 {
                print!(" Use32");
            }

            println!(" Length {}", seg.length);
            
            self.segments.push(seg.clone());
        }

        Ok(())
    }

    fn grpdef(&mut self, name: usize, segs: &Vec<usize>) -> Result<(), AppError> {
        println!("GRPDEF {}", self.lname(name));

        for segidx in segs.iter() {
            let seg = &self.segments[*segidx];
            println!("      {}", self.segname(seg)); 
        }

        self.groups.push(self.lname(name).to_string());
        Ok(())
    }

    fn extdef(&mut self, externs: &[Extern]) -> Result<(), AppError> {
        println!("EXTDEF");
        for ext in externs.iter() {
            println!("{:5} {} {}", self.externs.len(), ext.name, ext.typeidx);
            self.externs.push(ext.name.clone());
        }
        
        Ok(())
    }

    fn pubdef(&self, group: Option<usize>, seg: Option<usize>, frame: Option<u16>, publics: &[Public]) -> Result<(), AppError> {
        println!("PUBDEF");

        if let Some(group) = group {
            print!(" GRP={}", self.lname(group));
        }

        if let Some(seg) = seg {
            let seg = &self.segments[seg];
            print!(" SEG={}", self.segname(seg));
        }

        if let Some(frame) = frame {
            print!(" FRAME=${:04x}", frame);
        }

        println!();

        for public in publics {
            println!("      {:08x} {}", public.offset, public.name);
        }

        Ok(())
    }

    fn modend(&self, main: bool, start_address: Option<StartAddress> ) -> Result<(), AppError> {
        print!("MODEND");
        if main {
            print!(" MAIN");
        }
        println!();

        if let Some(sa) = start_address {
            print!("  Start address ");
            if let Some(fmethod) = sa.fmethod()? {
                print!("{:?} ", fmethod);
                if let Some(datum) = sa.frame_datum {
                    print!(" datum {} ", datum);
                }
            }

            if let Some(tmethod) = sa.tmethod()? {
                print!(" {:?}", tmethod);
                if let Some(datum) = sa.target_datum {
                    print!(" datum {}", datum);
                }
            }
        }
        println!();

        Ok(())
    }

    fn coment(&self, header: ComentHeader, coment: &Coment) -> Result<(), AppError> {
        print!("COMENT");
        if header.nopurge() {
            print!(" NOPURGE");
        }
        if header.nolist() {
            print!(" NOLIST");
        }
        println!();

        match coment {
            Coment::Translator{ text } => println!("  Translator '{}'", text),
            Coment::NewOMF{ text } => println!("  Debug style '{}'", text),
            Coment::MemoryModel{ text } => println!("  Memory model '{}'", text),
            Coment::DosSeg => println!("  DOS Segment order"),
            Coment::DefaultLibrary{ name } => println!("  Default library '{}'", name),
            Coment::Libmod{ name} => println!("  Libmod '{}'", name),
            Coment::User{ text } => println!("  User '{}'", text),

            _ => println!("  Unknown comment class {:02x}", header.comclass),
        }

        Ok(())
    }

    fn ledata(&self, seg: usize, offset: u32, data: &[u8]) -> Result<(), AppError> {
        let seg = &self.segments[seg];
        println!("LEDATA {}", self.segname(seg));

        let mut i = 0;

        while i < data.len() {
            const PERLINE: usize = 16;
            
            let mut left = data.len() - i;
            if left > PERLINE {
                left = PERLINE;
            }

            print!("      {:08x}", offset as usize + i);
            
            let mut j = 0;

            while j < left {
                print!(" {:02x}", data[i+j]);
                j += 1;
            }

            while j < PERLINE {
                print!("   ");
                j += 1;
            }

            print!(" |");

            j = 0;
            while j < left {
                let ch = data[i+j];
                if ch >= 0x20 && ch <= 0x7e {
                    if let Ok(s) = str::from_utf8(&data[i+j..i+j+1]) {
                        print!("{}", s);
                    } else {
                        print!("?");
                    }
                } else {
                    print!(".");
                }
                j += 1;
            }

            while j < PERLINE {
                print!(" ");
                j += 1;
            }

            println!("|");

            i += left;
        }
        Ok(())
    }

    fn comdef(&mut self, commons: &[Comdef]) -> Result<(), AppError> {
        println!("COMDEF");
        for com in commons.iter() {
            println!("{:5} {} Type={:02x} Length={}", self.externs.len(), com.name, com.datatype, com.length);
            self.externs.push(com.name.clone());
        }
        Ok(())
    }

    fn bakpat(&self, seg: usize, location: BakpatLocation, fixups: &[BakpatFixup]) -> Result<(), AppError> {
        println!("BAKPAT {} {:?}", self.segname(&self.segments[seg]), location);

        for fixup in fixups {
            println!("      Offset {:08x} Value {:08x}", fixup.offset, fixup.value);
        }

        Ok(())
    }

    fn fixupp(&self, fixups: &[FixupSubrecord]) -> Result<(), AppError> {
        println!("FIXUPP");

        for fixup in fixups {
            match fixup {
                FixupSubrecord::TargetThread{ method, thread, index } => {
                    print!("      TARGET THREAD {} {:?} ", thread, method);
                    match method {
                        TargetMethod::Segdef => print!("{}", self.segname(&self.segments[*index])),
                        TargetMethod::Grpdef => print!("{}", self.groups[*index]),
                        TargetMethod::Extdef => print!("{}", self.externs[*index]),
                        _ => (),
                    }
                    println!();
                },
                FixupSubrecord::FrameThread{ method, thread, index } => {
                    print!("      FRAME THREAD {} {:?} ", thread, method);
                    if let Some(index) = index {
                        match method {
                            FrameMethod::Segdef => print!("{}", self.segname(&self.segments[*index])),
                            FrameMethod::Grpdef => print!("{}", self.groups[*index]),
                            FrameMethod::Extdef => print!("{}", self.externs[*index]),
                            _ => (),
                        }
                    }
                    println!();
                },
                FixupSubrecord::Fixup{ fixup } => {
                    print!("      {:08x} {:?} ", fixup.data_offset, fixup.location);

                    if fixup.is_seg_relative {
                        print!("SEG-REL  ");
                    } else {
                        print!("SELF-REL ");
                    }

                    if let Some(ft) = fixup.frame_thread {
                        print!("FRAME-THREAD {} ", ft);
                    }

                    if let Some(fm) = fixup.frame_method.as_ref() {
                        match fm {
                            //
                            // TODO should refactor to put index into FrameMethod enum
                            //
                            FrameMethod::Segdef => print!("FRAME SEG {} ", self.segname(&self.segments[fixup.frame_datum.unwrap()])),
                            FrameMethod::Grpdef => print!("FRAME GROUP {} ", self.groups[fixup.frame_datum.unwrap()]),
                            FrameMethod::Extdef => print!("FRAME EXTERN {} ", self.externs[fixup.frame_datum.unwrap()]),
                            FrameMethod::Target => print!("FRAME=TARGET "),
                            FrameMethod::PreviousDataRecord => print!("FRAME=PREVIOUS-DATA-RECORDS "),
                        }
                    }

                    if let Some(tt) = fixup.target_thread {
                        print!("TARGET-THREAD {} ", tt);
                    }

                    if let Some(tm) = fixup.target_method.as_ref() {
                        match tm {
                            TargetMethod::Extdef | TargetMethod::ExtdefNoDisplacement =>
                                print!("TARGET EXTERN {} ", self.externs[fixup.target_datum.unwrap()]),
                            TargetMethod::Segdef | TargetMethod::SegdefNoDisplacement =>
                                print!("TARGET SEG {} ", self.segname(&self.segments[fixup.target_datum.unwrap()])),
                            TargetMethod::Grpdef | TargetMethod::GrpdefNoDisplacement =>
                                print!("TARGET GROUP {} ", self.groups[fixup.target_datum.unwrap()]),
                        }
                    }

                    println!("TARGET-DISP {}", fixup.target_displacement);
                },
            }
        }

        Ok(())
    }

    fn alias(&self, aliases: &[Alias]) -> Result<(), AppError> {
        println!("ALIAS");

        for alias in aliases {
            println!("  {} -> {}", alias.alias, alias.substitute);
        }

        Ok(())
    }
}

fn dump_one_object(obj: &[u8]) -> Result<(), AppError> {
    let mut obj = Parser::new(&obj);
    let mut objdump = Objdump::new();
    loop {
        match obj.next()? {
            Record::THEADR{ name } => println!("THEADER {}", name),
            Record::MODEND{ main, start_address } => objdump.modend(main, start_address)?,
            Record::LNAMES{ names } => objdump.lnames(&names)?,
            Record::SEGDEF{ segs } => objdump.segdef(&segs)?,
            Record::GRPDEF{ name, segs } => objdump.grpdef(name, &segs)?,
            Record::EXTDEF{ externs } => objdump.extdef(&externs)?,
            Record::PUBDEF{ group, seg, frame, publics} => objdump.pubdef(group, seg, frame, &publics)?,
            Record::COMENT{ header, coment } => objdump.coment(header, &coment)?,
            Record::LEDATA{ seg, offset, data } => objdump.ledata(seg, offset, &data)?,
            Record::BAKPAT{ seg, location, fixups} => objdump.bakpat(seg, location, &fixups)?,
            Record::FIXUPP{ fixups} => objdump.fixupp(&fixups)?,
            Record::COMDEF{ commons } => objdump.comdef(&commons)?,
            Record::LEXTDEF{ externs } => objdump.extdef(&externs)?,
            Record::ALIAS{ aliases } => objdump.alias(&aliases)?,
            Record::None => break,
            x => { println!("record {:x?}", x)},
        }
    }

    Ok(())
}

fn objdump() -> Result<(), AppError> {
    let args = Args::parse()?;
    let obj = std::fs::read(args.libname)?;

    if libfile::Parser::is_lib(&obj) {
        println!("FILE IS A LIBRARY");
        let mut lib = libfile::Parser::new(&obj)?;
        let mut obj = lib.first_obj()?;

        loop {
            match obj {
                None => break,
                Some(obj) => dump_one_object(obj)?,
            }

            obj = lib.next_obj()?;
            println!("--------------------");
        }
    } else {
        dump_one_object(&obj)?;
    }

    Ok(())
} 

fn main() {
    if let Err(err) = objdump() {
        println!("{}", err);
    }
}
