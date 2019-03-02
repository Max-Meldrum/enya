use crate::error::ErrorKind::*;
use crate::error::*;
use libc::c_int;

#[cfg(target_os = "linux")]
pub fn clock_ticks() -> Result<u64> {
    let id = libc::_SC_CLK_TCK as isize;
    let res = unsafe { libc::sysconf(id as c_int) };
    if res == -1 {
        Err(Error::new(InvalidData))
    } else {
        Ok(res as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticks() {
        let t = clock_ticks();
        assert_eq!(true, t.is_ok());
    }
}
