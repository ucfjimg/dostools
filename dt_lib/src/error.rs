use std::error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub struct Error {
    pub details: String,
}

// Format error for display
//
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl error::Error for Error{}

impl Error {
    // Create an error with an arbitrary message
    //
    pub fn new(details: &str) -> Error {
        Error {
            details: details.to_string(),
        }
    }

    pub fn truncated() -> Error {
        Error{
            details: "record is truncated".to_string(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::new(&format!("{}", err))
    }
}