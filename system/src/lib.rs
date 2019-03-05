extern crate bytes;
extern crate kompact;
#[macro_use]
extern crate slog;
extern crate api;
extern crate caps;

mod error;
mod monitor;

use caps::{CapSet, Capability};
use kompact::default_components::DeadletterBox;
use kompact::prelude::*;
use oci::Spec;
use std::fs::File;
use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};

use crate::error::ErrorKind::*;
use crate::error::*;

const CGROUPS_PATH: &str = "/sys/fs/cgroup";
const SYSTEM_HOST: &str = "127.0.0.1";
const SYSTEM_PORT: u16 = 2000;
const DEFAULT_INTERFACE: &str = "eth0";
const MONITOR_CGROUP: &str = "process";

pub struct System {
    linux_spec: Spec,
    cgroups_path: String,
    system: KompicsSystem,
}

impl System {
    #[cfg(target_os = "linux")]
    pub fn new(spec: Spec, cpath: Option<String>) -> Result<System> {
        let path = cpath.unwrap_or_else(|| String::from(CGROUPS_PATH));

        let _ = System::check_cgroups(path.clone())
            .map_err(|e| Error::with_cause(ReadFailed, e));

        Ok(System {
            linux_spec: spec,
            cgroups_path: path,
            system: System::system_setup(),
        })
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
        let ip_addr = SYSTEM_HOST
            .parse()
            .unwrap_or_else(|_| IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));

        let socket_addr = SocketAddr::new(ip_addr, SYSTEM_PORT);
        let mut cfg = KompicsConfig::new();

        cfg.label(String::from("System"));

        cfg.system_components(DeadletterBox::new, move || {
            let net_config = NetworkConfig::new(socket_addr);
            NetworkDispatcher::with_config(net_config)
        });

        KompicsSystem::new(cfg)
    }

    pub fn start(self) {
        info!(
            self.system.logger(),
            "Starting System at {}:{}", SYSTEM_HOST, SYSTEM_PORT
        );

        let cpath = self.cgroups_path.clone();
        let monitor = self.system.create_and_register(move || {
            let interface = if net::find_interface(DEFAULT_INTERFACE) {
                Some(String::from(DEFAULT_INTERFACE))
            } else {
                None // might be that eth0 has not "spawned" yet
            };

            monitor::Monitor::new(
                cpath,
                MONITOR_CGROUP.to_string(),
                interface,
                None, // Use default timeout
            )
        });

        let _ = self
            .system
            .register_by_alias(&monitor, "monitor")
            .await_timeout(std::time::Duration::from_millis(250))
            .expect("Failed to register enya monitor");

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
    fn system_group() {}
}
