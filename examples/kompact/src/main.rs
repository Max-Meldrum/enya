extern crate kompact;
extern crate bytes;
extern crate api;
#[macro_use]
extern crate slog;

use api::kompact_api::*;
use kompact::*;
use kompact::default_components::DeadletterBox;
use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

fn main() {
    // Default Enya setup
    let monitor_path_str: &str = "tcp://127.0.0.1:2000/monitor";

    let system_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let sub_addr_socket = SocketAddr::new(system_addr, 1500);
    let mut cfg = KompicsConfig::new();
    cfg.label(String::from("Subscriber"));
    cfg.system_components(DeadletterBox::new, move || {
        let net_config = NetworkConfig::new(sub_addr_socket);
        NetworkDispatcher::with_config(net_config)
    });

    let system = KompicsSystem::new(cfg);
    let enya_actor_path = ActorPath::from_str(monitor_path_str).unwrap();

    let (subscriber, _s) = system.create_and_register(move || {
        Subscriber::new(enya_actor_path)
    });

    system.start(&subscriber);

    std::thread::park();
}


#[derive(ComponentDefinition)]
pub struct Subscriber {
    ctx: ComponentContext<Subscriber>,
    enya_monitor: ActorPath,
}

impl Subscriber {
    pub fn new(path: ActorPath) -> Subscriber {
        Subscriber {
            ctx: ComponentContext::new(),
            enya_monitor: path,
        }
    }
}

impl Actor for Subscriber {
    fn receive_local(&mut self, _sender: ActorRef, msg: Box<Any>) {
    }
    fn receive_message(&mut self, sender: ActorPath, _ser_id: u64, buf: &mut Buf) {
        let result: Result<api::MetricReport, SerError> = ProtoSer::deserialise(buf);
        if let Ok(report) = result {
            info!(self.ctx.log(), "MetricReport from {:?}: {:?}", self.enya_monitor, report);
        } else {
            error!(self.ctx.log(), "Got unexpected message from {}", sender);
        }
    }
}

impl Provide<ControlPort> for Subscriber {
    fn handle(&mut self, event: ControlEvent) {
        if let ControlEvent::Start = event {
            let msg = api::Subscribe::new();
            self.enya_monitor.tell(msg, self);
        }
    }
}
