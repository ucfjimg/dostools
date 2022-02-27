mod args;
mod linker;

use std::fs;

use dt_lib::objfile::*;
use dt_lib::libfile;
use dt_lib::error::Error as AppError;

use crate::args::Args;
use crate::linker::{Objects};


fn read_objects(args: &Args) -> Result<Objects, AppError> {
    let mut objects = Objects::new();
    for objname in args.objs.iter() {
        let obj = fs::read(&objname)?;

        if libfile::Parser::is_lib(&obj) {
            objects.add_library(&objname, obj);
        } else {
            objects.add_object(&objname, obj);
        }
    }

    Ok(objects)
}

fn build_segments(objects: &Objects) -> Result<(), AppError> {
    for obj in objects.objs.iter() {
        let mut parser = Parser::new(&obj.image);

        loop {
            match parser.next()? {
                _ => (),
            }
        }
    }

    Ok(())

}

fn main() -> Result<(), AppError> {
    let args = Args::parse()?;

    let mut objects = read_objects(&args)?;

    build_segments(&mut objects)?;


    Ok(())
}
