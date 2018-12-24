extern crate kompact;
extern crate bytes;

#[macro_use]
extern crate slog;

mod monitor;

pub use kompact::prelude::*;
use kompact::default_components::DeadletterBox;
use std::net::SocketAddr;

const CGROUPS_PATH: &str = "/sys/fs/cgroup/";
const SCOUT_HOST: &str = "127.0.0.1";
const SCOUT_PORT: u16 = 2000;


pub struct Scout {
    cgroups_path: String,
    system: KompicsSystem,
}

impl Scout {

    #[cfg(target_os = "linux")]
    pub fn new(cgroups_path: Option<String>) -> Scout {
        let path = cgroups_path.unwrap_or_else(|| String::from(CGROUPS_PATH));
        Scout {
            cgroups_path: path,
            system: Scout::system_setup(),
        }
    }

    fn system_setup() -> KompicsSystem {
        let socket_addr = SocketAddr::new(SCOUT_HOST.parse().unwrap(), SCOUT_PORT);
        let mut cfg = KompicsConfig::new();

        cfg.system_components(DeadletterBox::new, move || {
            let net_config = NetworkConfig::new(socket_addr);
            NetworkDispatcher::with_config(net_config)
        });
        
        KompicsSystem::new(cfg)
    }

    pub fn start(self) {
        let monitor = self.system.create_and_register(monitor::Monitor::new);
        self.system.start(&monitor);
    }

    pub fn shutdown(self) {
        self.system.shutdown().expect("Could not exit properly")
    }
}

#[cfg(test)]
mod tests {}
