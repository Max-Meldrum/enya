use std::fs::File;
use std::io::Read;

use crate::error::*;
use crate::error::ErrorKind::*;

fn read_string_from(mut file: File) -> Result<String> {
    let mut string = String::new();
    match file.read_to_string(&mut string) {
        Ok(_) => Ok(string.trim().to_string()),
        Err(e) => Err(Error::with_cause(ReadFailed, e)),
    }
}

pub fn read_u64_from(path: &str) -> Result<u64> {
    let file = File::open(path);
    match file {
        Ok(mut f) => {
            let mut string = String::new();
            match f.read_to_string(&mut string) {
                Ok(_) => string.trim().parse().map_err(|e| Error::with_cause(ParseError, e)),
                Err(e) => Err(Error::with_cause(ReadFailed, e)),
            }
        }
        Err(e) => Err(Error::with_cause(InvalidPath, e)),
    }
}
