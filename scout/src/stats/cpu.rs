use crate::util;
use std::cell::Cell;

use std::fs::File;
use std::io::Read;

use std::io::{BufRead, BufReader};

use crate::error::*;
use crate::error::ErrorKind::*;

const CPUACCT_STAT: &str = "cpu/cpuacct.stat";

#[derive(Debug)]
pub struct Cpu {
    cgroups_path: String,
    user: Cell<u64>,
    system: Cell<u64>,
}

impl Cpu {
    pub fn new(path: String) -> Cpu {
        Cpu {
            cgroups_path: path,
            user: Cell::new(0),
            system: Cell::new(0),
        }
    }
    pub fn update(&mut self) {
        let cpuacct_stat_path = &(self.cgroups_path.to_owned() + CPUACCT_STAT);
        let res = Cpu::parse_cpuacct(cpuacct_stat_path);
    }


    fn parse_cpuacct(path: &str) -> Result<(u64, u64)> {
        //TODO
        Ok((1,1))
    }
}
