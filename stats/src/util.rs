use std::fs::File;
use std::io::Read;

use crate::error::ErrorKind::*;
use crate::error::*;

pub fn read_string_from(path: &str) -> Result<String> {
    match File::open(path) {
        Ok(mut f) => {
            let mut string = String::new();
            match f.read_to_string(&mut string) {
                Ok(_) => Ok(string.trim().to_string()),
                Err(e) => Err(Error::with_cause(ReadFailed, e)),
            }
        }
        Err(e) => Err(Error::with_cause(InvalidPath, e)),
    }
}

pub fn read_u64_from(path: &str) -> Result<u64> {
    match File::open(path) {
        Ok(mut f) => {
            let mut string = String::new();
            match f.read_to_string(&mut string) {
                Ok(_) => string
                    .trim()
                    .parse()
                    .map_err(|e| Error::with_cause(ParseError, e)),
                Err(e) => Err(Error::with_cause(ReadFailed, e)),
            }
        }
        Err(e) => Err(Error::with_cause(InvalidPath, e)),
    }
}

pub fn fmt_float(f: f64) -> Result<f64> {
    format!("{:.2}", f)
        .parse::<f64>()
        .map_err(|_| Error::new(ParseError))
}
