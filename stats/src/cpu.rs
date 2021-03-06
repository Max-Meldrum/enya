use crate::util;

use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::error::ErrorKind::*;
use crate::error::*;
use crate::sysconf::*;

lazy_static! {
    static ref CLOCK_TICKS: u64 =
        clock_ticks().expect("Unable to fetch clock ticks from sysconf");
}

const CPUACCT_USAGE: &str = "cpuacct.usage";
const CPUACCT_USAGE_PERCPU: &str = "cpuacct.usage_percpu";
const NANO_PER_SEC: u64 = 1_000_000_000;

#[derive(Debug)]
pub struct Cpu {
    cgroups_path: String,
    total_usage_path: String,
    per_cpu_path: String,
    pub total_usage: u64,
    pub system_usage: u64,
    pub avg: f64,
    collections: u64,
}

impl Cpu {
    pub fn new(path: String) -> Cpu {
        let total_usage_path = path.to_owned() + CPUACCT_USAGE;
        let per_cpu_path = path.to_owned() + CPUACCT_USAGE_PERCPU;
        Cpu {
            cgroups_path: path,
            total_usage_path,
            per_cpu_path,
            total_usage: 0,
            system_usage: 0,
            avg: 0.0,
            collections: 0,
        }
    }
    pub fn update(&mut self) {
        let total_usage = util::read_u64_from(&self.total_usage_path);

        if let Ok(usage) = total_usage {
            let mut cpu_percent = 0.0;

            if let Ok(sys) = self.get_system_cpu_usage() {
                let cpu_delta = usage as f64 - self.total_usage as f64;
                let system_delta = sys as f64 - self.system_usage as f64;

                if cpu_delta > 0.0 && system_delta > 0.0 {
                    let per_cpu_len =
                        self.get_per_cpu_usage().map(|v| v.len()).unwrap_or(0);
                    let percent =
                        (cpu_delta / system_delta) * per_cpu_len as f64 * 100.0;
                    if let Ok(res) = util::fmt_float(percent) {
                        cpu_percent = res;
                    }
                }

                // NOTE: we skip putting at 0 as the collections
                //       depend on known total_usage and system_usage data
                if self.collections == 1 {
                    self.avg = cpu_percent;
                } else {
                    let ema: f64 = (cpu_percent - self.avg)
                        * (2.0 / (self.collections + 1) as f64)
                        + self.avg;

                    if let Ok(ema_fmt) = util::fmt_float(ema) {
                        self.avg = ema_fmt;
                    }
                }

                self.collections += 1;
                self.total_usage = usage;
                self.system_usage = sys;
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
        let line = util::read_string_from(&self.per_cpu_path)?;
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
    const CGROUPS_PATH: &str = "/sys/fs/cgroup/cpu/";

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

    #[test]
    fn avg_cpu() {
        let mut cpu = Cpu::new(CGROUPS_PATH.to_string());
        let _ = cpu.update();
        std::thread::sleep(std::time::Duration::from_millis(200));
        let _ = cpu.update();
        std::thread::sleep(std::time::Duration::from_millis(200));
        let _ = cpu.update();
        std::thread::sleep(std::time::Duration::from_millis(200));
        assert!(cpu.avg > 0.0);
    }
}
