extern crate bytes;
extern crate kompact;

#[macro_use]
extern crate slog;

#[macro_use]
extern crate lazy_static;

mod error;
mod monitor;
mod stats;
mod sysconf;
mod util;
mod net;

use kompact::default_components::DeadletterBox;
pub use kompact::prelude::*;
pub use lazy_static::*;
use std::fs::File;
use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};
use caps::{CapSet, Capability};

use crate::net::*;
use crate::error::*;
use crate::error::ErrorKind::*;

const CGROUPS_PATH: &str = "/sys/fs/cgroup/";
const SCOUT_HOST: &str = "127.0.0.1";
const SCOUT_PORT: u16 = 2000;

pub struct Scout {
    cgroups_path: String,
    system: KompicsSystem,
}

impl Scout {
    #[cfg(target_os = "linux")]
    pub fn new(cgroups_path: Option<String>) -> Result<Scout> {
        let path = cgroups_path.unwrap_or_else(|| String::from(CGROUPS_PATH));

        let _ = Scout::check_cgroups(path.clone())
            .map_err(|e| Error::with_cause(CgroupsReadError, e));

        if !Scout::is_net_admin() {
            Err(Error::new(NetAdminError))
        } else {
            Ok(Scout {
                cgroups_path: path,
                system: Scout::system_setup(),
            })
        }

    }

    fn check_cgroups(path: String) -> std::io::Result<()> {
        let file = File::open(path + "memory/memory.stat")?;
        let meta = file.metadata()?;
        assert_eq!(meta.permissions().readonly(), true);
        Ok(())
    }

    fn is_net_admin() -> bool {
        let net_admin =
            caps::has_cap(None, CapSet::Permitted, Capability::CAP_NET_ADMIN);
        net_admin.unwrap_or(false)
    }

    fn system_setup() -> KompicsSystem {
        let ip_addr = SCOUT_HOST
            .parse()
            .unwrap_or_else(|_| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

        let socket_addr = SocketAddr::new(ip_addr, SCOUT_PORT);
        let mut cfg = KompicsConfig::new();

        cfg.label(String::from("Scout"));

        cfg.system_components(DeadletterBox::new, move || {
            let net_config = NetworkConfig::new(socket_addr);
            NetworkDispatcher::with_config(net_config)
        });

        KompicsSystem::new(cfg)
    }

    pub fn start(self) {
        info!(
            self.system.logger(),
            "Starting Scout at {}:{}", SCOUT_HOST, SCOUT_PORT
        );

        let monitor_path = self.cgroups_path.clone();
        let monitor = self.system.create_and_register(move || {
            monitor::Monitor::new(monitor_path, None) // TODO: find veth interface
        });

        self.system.start(&monitor);
    }

    pub fn shutdown(self) {
        self.system.shutdown().expect("Could not exit properly")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scout_setup() {
        let scout = Scout::new(None); // Assume default cgroups path
                                      //scout.unwrap().start();
    }
}
