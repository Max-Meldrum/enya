use api::protobuf::Message;
use bytes::Buf;
use kompact::prelude::BufMut;
use kompact::*;
use std::time::Duration;

use api::kompact_api::*;

use stats::cpu::Cpu;
use stats::io::*;
use stats::memory::*;
use stats::network::*;

const DEFAULT_TIMEOUT_MS: u64 = 2000;

#[derive(Clone, Copy)]
struct Collect {}

pub struct ProtoSer;

impl Deserialiser<api::Subscribe> for ProtoSer {
    fn deserialise(buf: &mut Buf) -> Result<api::Subscribe, SerError> {
        let parsed = api::protobuf::parse_from_bytes(buf.bytes())
            .map_err(|err| SerError::InvalidData(err.to_string()))?;
        Ok(parsed)
    }
}

#[derive(Clone, Debug)]
struct Subscribe(pub api::Subscribe);

impl Subscribe {
    pub fn new() -> Subscribe {
        Subscribe {
            0: api::Subscribe::new(),
        }
    }
}

impl Serialisable for Box<Subscribe> {
    fn serid(&self) -> u64 {
        serialisation_ids::PBUF
    }
    fn size_hint(&self) -> Option<usize> {
        if let Ok(bytes) = self.0.write_to_bytes() {
            Some(bytes.len())
        } else {
            None
        }
    }
    fn serialise(&self, buf: &mut BufMut) -> Result<(), SerError> {
        let bytes = self
            .0
            .write_to_bytes()
            .map_err(|err| SerError::InvalidData(err.to_string()))?;
        buf.put_slice(&bytes);
        Ok(())
    }
    fn local(self: Box<Self>) -> Result<Box<Any + Send>, Box<Serialisable>> {
        Ok(self)
    }
}

#[derive(Clone, Debug)]
struct Report(pub api::MetricReport);

impl Report {
    pub fn new() -> Report {
        Report {
            0: api::MetricReport::new(),
        }
    }
    pub fn set_id(&mut self, id: &str) {
        self.0.set_id(id.to_string());
    }
    pub fn set_memory(&mut self, memory: &Memory) {
        let mut mem = api::Memory::new();
        mem.set_usage(memory.usage);
        mem.set_limit(memory.limit);
        self.0.set_memory(mem);
    }

    pub fn set_cpu(&mut self, cpu: &Cpu) {
        let mut c = api::Cpu::new();
        c.set_total(cpu.total_usage);
        c.set_system(cpu.system_usage);
        self.0.set_cpu(c);
    }

    pub fn set_network(&mut self, net: &Network) {
        let mut network = api::Network::new();
        network.set_tx_bytes(net.tx_bytes);
        network.set_tx_packets(net.tx_packets);
        network.set_rx_bytes(net.rx_bytes);
        network.set_rx_packets(net.rx_packets);
        self.0.set_network(network);
    }
    pub fn set_io(&mut self, io: &Io) {
        let mut io_obj = api::Io::new();
        io_obj.set_read(io.read);
        io_obj.set_write(io.write);
        self.0.set_io(io_obj);
    }
}

impl Deserialiser<api::MetricReport> for ProtoSer {
    fn deserialise(buf: &mut Buf) -> Result<api::MetricReport, SerError> {
        let parsed = api::protobuf::parse_from_bytes(buf.bytes())
            .map_err(|err| SerError::InvalidData(err.to_string()))?;
        Ok(parsed)
    }
}
impl Serialisable for Box<Report> {
    fn serid(&self) -> u64 {
        serialisation_ids::PBUF
    }
    fn size_hint(&self) -> Option<usize> {
        if let Ok(bytes) = self.0.write_to_bytes() {
            Some(bytes.len())
        } else {
            None
        }
    }
    fn serialise(&self, buf: &mut BufMut) -> Result<(), SerError> {
        let bytes = self
            .0
            .write_to_bytes()
            .map_err(|err| SerError::InvalidData(err.to_string()))?;
        buf.put_slice(&bytes);
        Ok(())
    }
    fn local(self: Box<Self>) -> Result<Box<Any + Send>, Box<Serialisable>> {
        Ok(self)
    }
}

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
}

impl Monitor {
    pub fn new(
        path: String,
        interface: Option<String>,
        timeout: Option<u64>,
    ) -> Monitor {
        Monitor {
            ctx: ComponentContext::new(),
            timeout_ms: timeout.unwrap_or(DEFAULT_TIMEOUT_MS),
            collect_timer: None,
            cgroups_path: path.clone(),
            memory: Memory::new(path.clone()),
            cpu: Cpu::new(path.clone()),
            network: interface.and_then(|i| Some(Network::new(i))),
            io: Some(Io::new(path)),
            subscribers: Vec::new(),
        }
    }

    fn update(&mut self) {
        let mut report = Report::new();
        report.set_id("process");

        let _ = self.memory.update();
        self.cpu.update();
        debug!(self.ctx.log(), "Memory: {}%", self.memory.procentage);
        debug!(self.ctx.log(), "Cpu: {}%", self.cpu.percentage);

        report.set_memory(&self.memory);
        report.set_cpu(&self.cpu);

        if let Some(net) = self.network.as_mut() {
            net.update();
            report.set_network(net);
            debug!(self.ctx.log(), "Network: {:?}", net);
        }

        if let Some(io) = self.io.as_mut() {
            io.update();
            report.set_io(io);
            debug!(self.ctx.log(), "IO: {:?}", io);
        }

        // TODO: improve?
        for sub in self.subscribers.iter() {
            sub.tell(Box::new(report.clone()), self);
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
        let result: Result<api::Subscribe, SerError> = ProtoSer::deserialise(buf);

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
        fn receive_local(&mut self, _sender: ActorRef, msg: Box<Any>) {
        }
        fn receive_message(&mut self, sender: ActorPath, _ser_id: u64, buf: &mut Buf) {
            let result: Result<api::MetricReport, SerError> = ProtoSer::deserialise(buf);
            if let Ok(report) = result {
                self.reports_received += 1;
                info!(self.ctx.log(), "MetricReport: {:?}", report);
            } else {
                error!(self.ctx.log(), "Got unexpected message from {}", sender);
            }
        }
    }

    impl Provide<ControlPort> for Subscriber {
        fn handle(&mut self, event: ControlEvent) {
            if let ControlEvent::Start = event {
                let msg = Subscribe::new();
                self.target.tell(Box::new(msg),self);
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

        let subscriber = sub_system.create_and_register(move || {
            Subscriber::new(monitor_path)
        });

        sub_system.start(&subscriber);
        std::thread::sleep(std::time::Duration::from_millis(500));
        let sub = subscriber.definition().lock().unwrap();
        assert!(sub.reports_received >= 1);
    }
}

