use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub struct Error {
    pub details: String,
    pub offset: Option<usize>,
}

// Format error for display
//
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.offset {
            Some(offset) => write!(f, "{:08x}: {}", offset, self.details),
            None =>         write!(f, "{}", self.details),
        }
    }
}

impl error::Error for Error{}

impl Error {
    // Create an error with an arbitrary message
    //
    pub fn new(details: &str) -> Error {
        Error {
            details: details.to_string(),
            offset: None,
        }
    }

    pub fn with_offset(details: &str, offset: usize) -> Error {
        Error {
            details: details.to_string(),
            offset: Some(offset),
        }
    }

    pub fn truncated() -> Error {
        Error{
            details: "record is truncated".to_string(),
            offset: None,
        }
    }

}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::new(&format!("{}", err))
    }
}