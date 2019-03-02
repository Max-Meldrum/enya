extern crate libc;
#[macro_use]
extern crate lazy_static;

mod error;
mod sysconf;
mod util;

// Public interface
pub mod cpu;
pub mod io;
pub mod memory;
pub mod network;
