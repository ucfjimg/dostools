mod args;

use std::fs::File;
use std::io::Read;

use dt_lib::error::Error as AppError;
use dt_lib::record;
use crate::args::Args;

fn uint(data: &[u8]) -> usize {
    let bytes = data.len();
    let mut value: usize = 0;

    for i in 1..bytes+1 {
        let byte = data[bytes - i] as usize;
        value = (value << 8) | byte;
    }

    value
}

fn objdump() -> Result<(), AppError> {
    let args = Args::parse()?;
    let mut file = File::open(args.libname)?;

    let mut buf = [0u8; 3];

    loop {
        let size = file.read(&mut buf)?;
        if size == 0 {
            break;
        }

        if size < buf.len() {
            return Err(AppError::truncated());
        }

        let bytes = uint(&buf[1..3]);

        let mut data = Vec::new();
        data.resize(bytes, 0u8);

        let size = file.read(&mut data)?;
        if size < data.len() {
            return Err(AppError::truncated());
        }

        match record::RecordType::try_from(buf[0]) {
            Ok(rectype) => println!("{:?}", rectype),
            Err(_) => println!("unknown record type {:02x}", buf[0]),
        }
    }

    Ok(())
} 

fn main() {
    if let Err(err) = objdump() {
        println!("{}", err);
    }
}
