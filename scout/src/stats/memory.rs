use crate::util;
use std::cell::Cell;

const MEMORY_USAGE: &str = "memory/memory.usage_in_bytes";
const MEMORY_LIMIT: &str = "memory/memory.limit_in_bytes";

// LOW: >= 0 <= 30
// MEDIUM: > 30 <= 60
// HIGH: > 60 < 95
// CRITICAL: >= 95
const LOW: u16 = 30;
const MEDIUM: u16 = 60;
const HIGH: u16 = 80;

pub enum MemoryStatus {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug)]
pub struct Memory {
    cgroups_path: String,
    pub usage: Cell<u64>,
    pub limit: Cell<u64>,
    pub procentage: Cell<f32>,
}

impl Memory {
    pub fn new(path: String) -> Memory {
        Memory {
            cgroups_path: path,
            usage: Cell::new(0),
            limit: Cell::new(0),
            procentage: Cell::new(0.0),
        }
    }
    pub fn update(&mut self) -> MemoryStatus {
        let usage_path = &(self.cgroups_path.to_owned() + MEMORY_USAGE);
        let usage = util::read_u64_from(usage_path);
        let bad_res = 0;
        self.usage.set(usage.unwrap_or(bad_res));

        // Perhaps just read it once at start?
        // However, might change if container updates the limit
        let limit_path = &(self.cgroups_path.to_owned() + MEMORY_LIMIT);
        let limit = util::read_u64_from(limit_path);
        self.limit.set(limit.unwrap_or(bad_res));

        let mut mem_percent: f32 = 0.0;

        if self.limit.get() != 0 {
            let avg = self.usage.get() as f32 / self.limit.get() as f32;
            let f = format!("{:.2}", avg).parse::<f32>();
            if let Ok(v) = f {
                mem_percent = v;
            }
        }

        self.procentage.set(mem_percent);

        let level = (self.procentage.get() * 100 as f32) as u16;

        if level <= LOW {
            MemoryStatus::Low
        } else if level > LOW && level <= MEDIUM {
            MemoryStatus::Medium
        } else if level > MEDIUM && level <= HIGH {
            MemoryStatus::High
        } else {
            MemoryStatus::Critical
        }
    }
}
