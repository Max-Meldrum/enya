// Credit to https://github.com/levex/cgroups-rs/blob/master/src/error.rs

use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, Eq, PartialEq)]
pub enum ErrorKind {
    ReadFailed,
    InvalidPath,
    ParseError
}


#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    cause: Option<Box<StdError + Send>>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self.kind {
            ErrorKind::ReadFailed => "unable to read file",
            ErrorKind::ParseError => "unable to parse data",
            ErrorKind::InvalidPath => "bad path given",
        };
        write!(f, "{}", msg)
    }
}

impl StdError for Error {
    fn cause(&self) -> Option<&StdError> {
        match self.cause {
            Some(ref x) => Some(&**x),
            None => None,
        }
    }
}


impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            cause: None,
        }
    }

    pub(crate) fn with_cause<E>(kind: ErrorKind, cause: E) -> Self
    where
        E: 'static + Send + StdError,
    {
        Self {
            kind,
            cause: Some(Box::new(cause)),
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;
