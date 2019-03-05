use api::protobuf::Message;
use bytes::Buf;
use kompact::prelude::BufMut;
use kompact::*;
use std::time::Duration;

use api::kompact_api::ProtoSer;
use api::kompact_api::*;

use stats::cpu::Cpu;
use stats::io::*;
use stats::memory::*;
use stats::network::*;

const DEFAULT_TIMEOUT_MS: u64 = 2000;

#[derive(Clone, Copy)]
struct Collect {}

#[derive(ComponentDefinition)]
pub struct Monitor {
    ctx: ComponentContext<Monitor>,
    timeout_ms: u64,
    collect_timer: Option<ScheduledTimer>,
    cgroups_path: String,
    memory: Memory,
    cpu: Cpu,
    network: Option<Network>,
    io: Option<Io>,
    subscribers: Vec<ActorPath>,
    cgroup_name: String,
}

impl Monitor {
    pub fn new(
        path: String,
        cgroup_name: String,
        interface: Option<String>,
        timeout: Option<u64>,
    ) -> Monitor {
        let mem_path = format!("{}/memory/{}/", path, cgroup_name);
        let cpu_path = format!("{}/cpu/{}/", path, cgroup_name);
        let blkio_path = format!("{}/blkio/{}/", path, cgroup_name);

        Monitor {
            ctx: ComponentContext::new(),
            timeout_ms: timeout.unwrap_or(DEFAULT_TIMEOUT_MS),
            collect_timer: None,
            cgroups_path: path.clone(),
            memory: Memory::new(mem_path),
            cpu: Cpu::new(cpu_path),
            network: interface.and_then(|i| Some(Network::new(i))),
            io: Some(Io::new(blkio_path)),
            subscribers: Vec::new(),
            cgroup_name,
        }
    }

    fn create_report(&mut self) -> api::MetricReport {
        let mut report = api::MetricReport::new();
        report.set_id(String::from("process"));

        let mut mem = api::Memory::new();
        mem.set_usage(self.memory.usage);
        mem.set_limit(self.memory.limit);
        report.set_memory(mem);

        let mut cpu = api::Cpu::new();
        cpu.set_total(self.cpu.total_usage);
        cpu.set_system(self.cpu.system_usage);
        report.set_cpu(cpu);

        if let Some(net) = self.network.as_mut() {
            let mut network = api::Network::new();
            network.set_tx_bytes(net.tx_bytes);
            network.set_tx_packets(net.tx_packets);
            network.set_rx_bytes(net.rx_bytes);
            network.set_rx_packets(net.rx_packets);
            report.set_network(network);
        }

        if let Some(io) = self.io.as_mut() {
            let mut io_obj = api::Io::new();
            io_obj.set_read(io.read);
            io_obj.set_write(io.write);
            report.set_io(io_obj);
        }

        report
    }

    fn update(&mut self) {
        let _ = self.memory.update();
        self.cpu.update();
        debug!(self.ctx.log(), "Memory: {}%", self.memory.procentage);
        debug!(self.ctx.log(), "Cpu: {}%", self.cpu.percentage);

        if let Some(net) = self.network.as_mut() {
            net.update();
            debug!(self.ctx.log(), "Network: {:?}", net);
        }

        if let Some(io) = self.io.as_mut() {
            io.update();
            debug!(self.ctx.log(), "IO: {:?}", io);
        }

        let report = self.create_report();

        // TODO: improve?
        for sub in self.subscribers.iter() {
            sub.tell(report.clone(), self);
        }
    }

    fn stop_collect(&mut self) {
        if let Some(timer) = self.collect_timer.clone() {
            self.cancel_timer(timer);
            self.collect_timer = None;
        }
    }
}

impl Provide<ControlPort> for Monitor {
    fn handle(&mut self, event: ControlEvent) {
        match event {
            ControlEvent::Start => {
                let timeout = Duration::from_millis(self.timeout_ms);
                let timer =
                    self.schedule_periodic(timeout, timeout, |self_c, _| {
                        self_c.actor_ref().tell(Box::new(Collect {}), self_c);
                    });

                self.collect_timer = Some(timer);
            }
            ControlEvent::Stop => self.stop_collect(),
            ControlEvent::Kill => self.stop_collect(),
        }
    }
}

impl Actor for Monitor {
    fn receive_local(&mut self, _sender: ActorRef, msg: Box<Any>) {
        if let Ok(_collect) = msg.downcast::<Collect>() {
            self.update();
        }
    }
    fn receive_message(
        &mut self,
        sender: ActorPath,
        _ser_id: u64,
        buf: &mut Buf,
    ) {
        let result: Result<api::Subscribe, SerError> =
            ProtoSer::deserialise(buf);

        if let Ok(res) = result {
            debug!(self.ctx.log(), "Adding subscriber {}", sender);
            self.subscribers.push(sender);
        } else {
            error!(self.ctx.log(), "Got unexpected message from {}", sender);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kompact::default_components::DeadletterBox;
    use std::net::SocketAddr;
    use std::net::{IpAddr, Ipv4Addr};

    const SYSTEM_HOST: &str = "127.0.0.1";
    const SYSTEM_PORT: u16 = 2000;

    #[derive(ComponentDefinition)]
    pub struct Subscriber {
        ctx: ComponentContext<Subscriber>,
        target: ActorPath,
        pub reports_received: u64,
    }

    impl Subscriber {
        pub fn new(path: ActorPath) -> Subscriber {
            Subscriber {
                ctx: ComponentContext::new(),
                target: path,
                reports_received: 0,
            }
        }
    }

    impl Actor for Subscriber {
        fn receive_local(&mut self, _sender: ActorRef, msg: Box<Any>) {}
        fn receive_message(
            &mut self,
            sender: ActorPath,
            _ser_id: u64,
            buf: &mut Buf,
        ) {
            let result: Result<api::MetricReport, SerError> =
                ProtoSer::deserialise(buf);
            if let Ok(report) = result {
                self.reports_received += 1;
                info!(self.ctx.log(), "MetricReport: {:?}", report);
            } else {
                error!(
                    self.ctx.log(),
                    "Got unexpected message from {}", sender
                );
            }
        }
    }

    impl Provide<ControlPort> for Subscriber {
        fn handle(&mut self, event: ControlEvent) {
            if let ControlEvent::Start = event {
                let msg = api::Subscribe::new();
                self.target.tell(msg, self);
            }
        }
    }

    #[test]
    fn subscription_test() {
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

        let system = KompicsSystem::new(cfg);

        let monitor = system.create_and_register(move || {
            Monitor::new(
                String::from("/sys/fs/cgroup/"),
                "".to_string(),
                None,
                Some(250),
            )
        });

        let _ = system
            .register_by_alias(&monitor, "monitor")
            .await_timeout(Duration::from_millis(1000))
            .expect("Registration never completed.");

        system.start(&monitor);

        let sub_system_addr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let sub_addr_socket = SocketAddr::new(sub_system_addr, 1337);

        let mut cfg = KompicsConfig::new();
        cfg.label(String::from("Subscriber"));

        cfg.system_components(DeadletterBox::new, move || {
            let net_config = NetworkConfig::new(sub_addr_socket);
            NetworkDispatcher::with_config(net_config)
        });

        let sub_system = KompicsSystem::new(cfg);

        let monitor_path = ActorPath::Named(NamedPath::with_socket(
            Transport::TCP,
            socket_addr,
            vec!["monitor".into()],
        ));

        let subscriber = sub_system
            .create_and_register(move || Subscriber::new(monitor_path));

        sub_system.start(&subscriber);
        std::thread::sleep(std::time::Duration::from_millis(500));
        let sub = subscriber.definition().lock().unwrap();
        assert!(sub.reports_received >= 1);
    }
}
