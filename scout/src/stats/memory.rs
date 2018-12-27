use crate::util;
use std::cell::Cell;

const MEMORY_USAGE: &str = "memory/memory.usage_in_bytes";
const MEMORY_LIMIT: &str = "memory/memory.limit_in_bytes";

#[derive(Debug)]
pub struct Memory {
    pub usage: Cell<u64>,
    pub limit: Cell<u64>,
    pub procentage: Cell<f32>,
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            usage: Cell::new(0),
            limit: Cell::new(0),
            procentage: Cell::new(0.0),
        }
    }
    pub fn stats(&mut self, path: &str) {
        let usage_path = &(path.to_owned() + MEMORY_USAGE);
        let usage = util::read_u64_from(usage_path);
        let bad_res = 0;
        self.usage.set(usage.unwrap_or(bad_res));

        // Perhaps just read it once at start?
        // However, might change if container updates the limit
        let limit_path = &(path.to_owned() + MEMORY_LIMIT);
        let limit = util::read_u64_from(limit_path);
        self.limit.set(limit.unwrap_or(bad_res));

        let avg = self.usage.get() as f32 / self.limit.get() as f32;
        let p = format!("{:.2}", avg).parse::<f32>();
        self.procentage.set(p.unwrap_or(0.0));
    }
}
