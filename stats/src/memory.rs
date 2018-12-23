use crate::util;

const MEMORY_USAGE: &str = "memory/memory.usage_in_bytes";
const MEMORY_LIMIT: &str = "memory/memory.limit_in_bytes";

// LOW: >= 0 <= 30
// MEDIUM: > 30 <= 60
// HIGH: > 60 < 95
// CRITICAL: >= 95
const LOW: u16 = 30;
const MEDIUM: u16 = 60;
const HIGH: u16 = 95;

pub enum MemoryStatus {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug)]
pub struct Memory {
    cgroups_path: String,
    usage_path: String,
    limit_path: String,
    pub usage: u64,
    pub limit: u64,
    pub procentage: f32,
}

impl Memory {
    pub fn new(path: String) -> Memory {
        let up = path.to_owned() + MEMORY_USAGE;
        let lp = path.to_owned() + MEMORY_LIMIT;
        Memory {
            cgroups_path: path,
            usage_path: up,
            limit_path: lp,
            usage: 0,
            limit: 0,
            procentage: 0.0,
        }
    }
    pub fn update(&mut self) -> MemoryStatus {
        let usage = util::read_u64_from(&self.usage_path);
        self.usage = usage.unwrap_or(0);

        // Perhaps just read it once at start?
        // However, might change if container updates the limit
        let limit = util::read_u64_from(&self.limit_path);
        self.limit = limit.unwrap_or(0);

        let mut mem_percent: f32 = 0.0;

        if self.limit != 0 {
            let avg = self.usage as f32 / self.limit as f32 * 100.0;
            let f = format!("{:.2}", avg).parse::<f32>();
            if let Ok(v) = f {
                mem_percent = v;
            }
        }

        self.procentage = mem_percent;

        let level = self.procentage as u16;

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
