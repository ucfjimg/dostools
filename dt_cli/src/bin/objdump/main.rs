mod args;

use dt_lib::error::Error as AppError;
use dt_lib::record::{Record, RecordType};
use dt_lib::objfile::*;

use crate::args::Args;

struct Objdump {
    lnames: Vec<String>,
    segments: Vec<OmfSegment>,
}

impl Objdump {
    fn new() -> Objdump {
        Objdump {
            lnames: vec!["".to_string()],
            segments: vec![OmfSegment::empty()],
        }
    }

    fn theadr(&self, rec: &Record) -> Result<(), AppError> {
        let rec = OmfTheadr::new(rec)?;
        println!("THEADR {}", rec.name);
        Ok(())
    }
    
    fn lnames(&mut self, rec: &Record) -> Result<(), AppError> {
        let rec = OmfLnames::new(rec)?;

        println!("LNAMES");
        for name in rec.names.iter() {
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

    fn segdef(&mut self, rec: &Record) -> Result<(), AppError> {
        let is32 = rec.rectype.is32();
        let rec = OmfSegdef::new(rec)?;

        println!("{}", if is32 { "SEGDEF32" } else { "SEGDEF" });
        for seg in rec.omfsegs.iter() {
            print!("{:5} {}.{}.{} {:?} {:?}",
                self.segments.len(),
                self.lname(seg.class),
                self.lname(seg.name),
                self.lname(seg.overlay),
                seg.align,
                seg.combine,
            );
            
            if let Some(frame) = seg.frame {
                print!(" Frame=${:04x}", frame);
                if let Some(offset) = seg.offset {
                    print!(":{:04x}", offset);
                }                
            }

            if seg.use32 {
                print!(" Use32");
            }

            println!(" Length {}", seg.length);
            
            self.segments.push(seg.clone());
        }

        Ok(())
    }

    fn do_record(&mut self, rec: &Record) -> Result<(), AppError> {
        match rec.rectype {
            RecordType::THeader => self.theadr(rec),
            RecordType::LNames => self.lnames(rec),
            RecordType::SegDef | 
            RecordType::SegDef32 => self.segdef(rec),
            RecordType::Unknown{ typ } => Err(AppError::new(&format!("skipping unrecognized record {:02x}", typ))),
            _ => {
                println!("not yet supported {:?}", rec.rectype);
                Ok(())
            },
        }
    }
}

fn objdump() -> Result<(), AppError> {
    let args = Args::parse()?;
    let mut obj = ObjFile::read(&args.libname)?;
    let mut objdump = Objdump::new();

    loop {
        match obj.next()? {
            Some(rec) => 
                if let Err(e) = objdump.do_record(&rec) {
                    println!("parsing error {}", e);
                },
            None => break,
        }
    }
    Ok(())
} 




fn main() {
    if let Err(err) = objdump() {
        println!("{}", err);
    }
}
