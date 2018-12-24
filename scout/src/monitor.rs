use kompact::*;
use bytes::Buf;

#[derive(ComponentDefinition)]
pub struct Monitor {
    ctx: ComponentContext<Monitor>
}

impl Monitor {
    pub fn new() -> Monitor {
        Monitor {
            ctx: ComponentContext::new(),
        }
    }
}

impl Provide<ControlPort> for Monitor {
    fn handle(&mut self, event: ControlEvent) {
        if let ControlEvent::Start = event {
            info!(self.ctx.log(), "Starting Monitor");
        }
    }
}

impl Actor for Monitor {
    fn receive_local(&mut self, _sender: ActorRef, _msg: Box<Any>) {

    }
    fn receive_message(&mut self, sender: ActorPath, _ser_id: u64, _buf: &mut Buf) {
        error!(self.ctx.log(), "Got unexpected message from {}", sender);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kompact::default_components::DeadletterBox;
    use std::net::SocketAddr;

    #[test]
    fn setup() {
        let socket_addr = SocketAddr::new("127.0.0.1".parse().unwrap(), 2000);
        let mut cfg = KompicsConfig::new();

        cfg.system_components(DeadletterBox::new, move || {
            let net_config = NetworkConfig::new(socket_addr);
            NetworkDispatcher::with_config(net_config)
        });

        let system = KompicsSystem::new(cfg);

        let monitor = system.create(Monitor::new);

        system.start(&monitor);
    }
}
