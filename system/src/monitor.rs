use bytes::Buf;
use kompact::Timer;

use kompact::*;
use std::time::Duration;

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
            ControlEvent::Start =>  {
                let timeout = Duration::from_millis(self.timeout_ms);
                let timer =
                    self.schedule_periodic(timeout, timeout, |self_c, _| {
                        self_c.actor_ref().tell(Box::new(Collect {}), self_c);
                    });

                self.collect_timer = Some(timer);
            },
            ControlEvent::Stop => {
                self.stop_collect()
            },
            ControlEvent::Kill => {
                self.stop_collect()
            },
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
        _buf: &mut Buf,
    ) {

        // Subscription
        // Add sender to subscribers
        error!(self.ctx.log(), "Got unexpected message from {}", sender);
    }
}
