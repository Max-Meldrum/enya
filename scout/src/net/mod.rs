pub mod tc;
pub mod tower;

use std::process::Command;

use libc;
use std::ffi::CStr;
use std::mem;
use std::os::raw::c_char;
use std::str::from_utf8_unchecked;

pub fn tc_exists() -> bool {
    let cmd: &str = "tc";
    let tc_output = Command::new(cmd).arg("qdisc").arg("show").output();
    tc_output.is_ok()
}

pub fn find_interface(interface: &str) -> bool {
    unsafe {
        let mut addrs: *mut libc::ifaddrs = mem::uninitialized();
        if libc::getifaddrs(&mut addrs) != 0 {
            false
        } else {
            let mut info = addrs;
            let mut found = false;
            while !info.is_null() && !found {
                let c_str = (*info).ifa_name as *const c_char;
                let bytes = CStr::from_ptr(c_str).to_bytes();
                let name = from_utf8_unchecked(bytes).to_owned();
                if name == interface {
                    found = true;
                }
                info = (*info).ifa_next;
            }
            libc::freeifaddrs(addrs);
            found
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn tc_test() {
        assert_eq!(tc_exists(), true);
    }

    #[test]
    fn interface_test() {
        let interface: &str = "lo"; // check loopback as interfaces may differ on hosts
        assert_eq!(find_interface(interface), true);
    }
}
