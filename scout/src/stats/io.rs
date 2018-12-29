use crate::util;
use std::cell::Cell;

const MEMORY_USAGE: &str = "blkio/memory.usage_in_bytes";
const MEMORY_LIMIT: &str = "blkio/memory.limit_in_bytes";


#[derive(Debug)]
pub struct Io {
    cgroups_path: String,
    pub usage: Cell<u64>,
    pub limit: Cell<u64>,
    pub procentage: Cell<f32>,
}
