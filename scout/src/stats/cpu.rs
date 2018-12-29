use crate::util;
use std::cell::Cell;

use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::error::ErrorKind::*;
use crate::error::*;
use crate::sysconf::*;

lazy_static! {
    static ref CLOCK_TICKS: u64 =
        clock_ticks().expect("Unable to fetch clock ticks from sysconf");
}

const CPUACCT_USAGE: &str = "cpu/cpuacct.usage";
const CPUACCT_USAGE_PERCPU: &str = "cpu/cpuacct.usage_percpu";
const NANO_PER_SEC: u64 = 1_000_000_000;

#[derive(Debug)]
pub struct Cpu {
    cgroups_path: String,
    total_usage: Cell<u64>,
    system_usage: Cell<u64>,
    percentage: Cell<f64>,
}

impl Cpu {
    pub fn new(path: String) -> Cpu {
        Cpu {
            cgroups_path: path,
            total_usage: Cell::new(0),
            system_usage: Cell::new(0),
            percentage: Cell::new(0.0),
        }
    }
    pub fn update(&mut self) {
        let total_usage_path = &(self.cgroups_path.to_owned() + CPUACCT_USAGE);
        let total_usage = util::read_u64_from(total_usage_path);

        if let Ok(usage) = total_usage {
            let mut cpu_percent = 0.0;

            if let Ok(sys) = self.get_system_cpu_usage() {
                let cpu_delta = usage as f64 - self.total_usage.get() as f64;
                let system_delta = sys as f64 - self.system_usage.get() as f64;

                if cpu_delta > 0.0 && system_delta > 0.0 {
                    let per_cpu_len = match self.get_per_cpu_usage() {
                        Ok(vec) => vec.len(),
                        Err(_) => 0,
                    };
                    let percent =
                        (cpu_delta / system_delta) * per_cpu_len as f64 * 100.0;
                    let formatted = format!("{:.2}", percent).parse::<f64>();
                    if let Ok(res) = formatted {
                        cpu_percent = res;
                    }
                }
                self.percentage.set(cpu_percent);
                self.total_usage.set(usage);
                self.system_usage.set(sys);
            }
        }
    }

    fn get_system_cpu_usage(&self) -> Result<u64> {
        match File::open("/proc/stat") {
            Ok(file) => {
                let mut reader = BufReader::new(file);
                let mut line = String::new();
                let _ = reader
                    .read_line(&mut line)
                    .map_err(|e| Error::with_cause(ReadFailed, e));

                let mut fields = line.split_whitespace();
                let cpu_opt = fields.next();

                if let Some(field) = cpu_opt {
                    if field == "cpu" {
                        let count = fields.clone().count();
                        if count < 8 {
                            Err(Error::new(CpuParseError))
                        } else {
                            let ticks: Result<u64> = fields
                                .map(|x| {
                                    x.parse::<u64>().map_err(|e| {
                                        Error::with_cause(ParseError, e)
                                    })
                                })
                                .sum::<Result<u64>>()
                                .map(|x| (x * NANO_PER_SEC) / *CLOCK_TICKS);
                            ticks
                        }
                    } else {
                        Err(Error::new(CpuParseError))
                    }
                } else {
                    Err(Error::new(CpuParseError))
                }
            }
            Err(e) => Err(Error::with_cause(ReadFailed, e)),
        }
    }

    fn get_per_cpu_usage(&self) -> Result<Vec<u64>> {
        let path = &(self.cgroups_path.to_owned() + CPUACCT_USAGE_PERCPU);
        let line = util::read_string_from(path)?;
        line.split_whitespace()
            .map(|x| {
                x.parse::<u64>()
                    .map_err(|e| Error::with_cause(ParseError, e))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const CGROUPS_PATH: &str = "/sys/fs/cgroup/";

    #[test]
    fn cpu_usage() {
        let cpu = Cpu::new(CGROUPS_PATH.to_string());
        let res = Cpu::get_system_cpu_usage(&cpu);
        assert!(res.unwrap() > 0);
    }

    #[test]
    fn per_cpu() {
        let cpu = Cpu::new(CGROUPS_PATH.to_string());
        let res = Cpu::get_per_cpu_usage(&cpu);
        assert!(res.unwrap().len() > 0);
    }
}
