use std::env;

use dt_lib::error::Error as ArgError;

#[derive(Debug)]
pub struct Args {
    args: env::Args,
    arg: Option<String>,
    pub objs: Vec<String>,
}

impl Args {
    fn new() -> Args {
        Args{ 
            args: env::args(),
            arg: None,
            objs: Vec::new(),
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
                        _ => return Err(ArgError::new(&format!("invalid flag {}", flag))),
                    }
                            },
                None => break,
            }
        }

        loop {
            match args.arg {
                Some(ref name) => args.objs.push(name.clone()),
                None => break,
            }

            args.next();
        }

        Ok(args)
    }
}