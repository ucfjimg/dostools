use std::env;

use dt_lib::error::Error as ArgError;

#[derive(Debug)]
pub enum Operation {
    List,
}

#[derive(Debug)]
pub struct Args {
    pub op: Operation,
    pub libname: String,
    
    args: env::Args,
    arg: Option<String>,
}

impl Args {
    fn new() -> Args {
        Args{ 
            op: Operation::List,
            libname: "".to_string(),
            args: env::args(),
            arg: None,
        }
    }

    fn next(&mut self) {
        self.arg = self.args.next(); 
    }

    pub fn parse() -> Result<Args, ArgError> {
        let mut args = Args::new();
        
        // skip program name
        args.next();

        // flags
        loop {
            args.next();

            match args.arg.as_ref().map(|s| s.as_str()) {
                Some(flag) => if !flag.starts_with("-") {
                    break
                } else {
                    match flag {
                        "-t" => args.op = Operation::List,
                        _ => return Err(ArgError::new(&format!("invalid flag {}", flag))),
                    }
                            },
                None => break,
            }
        }

        match args.arg {
            Some(ref name) => args.libname = name.clone(),
            None => return Err(ArgError::new("missing library name")),
        }

        Ok(args)
    }
}