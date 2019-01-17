use crate::util;
use std::cell::Cell;

use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::error::ErrorKind::*;
use crate::error::*;

const BLKIO_SERVICE_BYTES: &str = "blkio/blkio.io_service_bytes";

#[derive(Debug)]
pub struct Io {
    cgroups_path: String,
    pub write: Cell<u64>,
    pub read: Cell<u64>,
}

impl Io {
    pub fn new(cgroups_path: String) -> Io {
        Io {
           cgroups_path,
           write: Cell::new(0),
           read: Cell::new(0),
        }
    }

    pub fn update(&self) {
        let path = &(self.cgroups_path.to_owned() + BLKIO_SERVICE_BYTES);
        if let Ok((read, write)) = Io::parse_blkio_stat(path) {
            self.read.set(read);
            self.write.set(write);
        }
    }

    // Refactor
    fn parse_blkio_stat(path: &str) -> Result<(u64, u64)> {
        match File::open(path) {
            Ok(file) => {
                let mut reader = BufReader::new(file);
                let mut line = String::new();
                let _ = reader
                    .read_line(&mut line)
                    .map_err(|e| Error::with_cause(ReadFailed, e));

                let read_vec: Vec<_> = line.split_whitespace().collect();
                if read_vec.len() > 2 {
                    if read_vec[1] == "Read" {
                        let bytes_read = read_vec[2].parse::<u64>()
                            .map_err(|e| Error::with_cause(ParseError, e));

                        line.clear();
                        let _ = reader
                            .read_line(&mut line)
                            .map_err(|e| Error::with_cause(ReadFailed, e));
                        
                        let write_vec: Vec<_> = line.split_whitespace().collect();
                        if write_vec.len() > 2 {
                            if write_vec[1] == "Write" {
                                let bytes_write = write_vec[2].parse::<u64>()
                                    .map_err(|e| Error::with_cause(ParseError, e));

                                // Return read, write
                                bytes_read.and_then(|r| bytes_write.and_then(|w| Ok((r,w))))
                            } else {
                                Err(Error::new(BlkioParseError))
                            }
                        } else {
                            Err(Error::new(BlkioParseError))
                        }
                    } else {
                        Err(Error::new(BlkioParseError))
                    }
                } else {
                    Err(Error::new(BlkioParseError))
                }
            }
            Err(e) => Err(Error::with_cause(ReadFailed, e)),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    const CGROUPS_PATH: &str = "/sys/fs/cgroup/";

    #[test]
    fn blkio_test() {
        let io = Io::new(CGROUPS_PATH.to_string());
        let res = Io::update(&io);
        assert!(io.read.get() > 0);
        assert!(io.write.get() > 0);
    }
}
