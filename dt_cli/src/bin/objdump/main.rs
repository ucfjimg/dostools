mod args;

use dt_lib::error::Error as AppError;
use dt_lib::record::{Record, RecordType};
use dt_lib::objfile::*;

use crate::args::Args;

fn theadr(rec: &Record) -> Result<(), AppError> {
    let rec = OmfTheadr::new(rec)?;
    println!("THEADR {}", rec.name);
    Ok(())
}

fn do_record(rec: &Record) -> Result<(), AppError> {
    match rec.rectype {
        RecordType::THeader => theadr(rec),
        RecordType::Unknown{ typ } => Err(AppError::new(&format!("skipping unrecognized record {:02x}", typ))),
        _ => {
            println!("not yet supported {:?}", rec.rectype);
            Ok(())
        },
    }
}

fn objdump() -> Result<(), AppError> {
    let args = Args::parse()?;
    let mut obj = ObjFile::read(&args.libname)?;

    loop {
        match obj.next()? {
            Some(rec) => 
                if let Err(e) = do_record(&rec) {
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
