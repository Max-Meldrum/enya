use crate::util;
use std::cell::Cell;

const BLKIO_SERVICED_RECURSIVE: &str = "blkio/blkio.io_serviced_recursive";


#[derive(Debug)]
pub struct Io {
    cgroups_path: String,
}

impl Io {
    pub fn new(cgroups_path: String) -> Io {
        Io {
           cgroups_path 
        }
    }
}
