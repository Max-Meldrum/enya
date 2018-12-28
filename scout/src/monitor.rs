use bytes::Buf;
use kompact::Timer;

use kompact::*;
use std::time::Duration;

use crate::stats::memory::*;
use crate::stats::network::*;
use crate::stats::cpu::Cpu;

#[derive(Clone, Copy)]
struct Collect {}


#[derive(ComponentDefinition)]
pub struct Monitor {
    ctx: ComponentContext<Monitor>,
    collect_timer: Option<ScheduledTimer>,
    cgroups_path: String,
    memory: Memory,
    cpu: Cpu,
    network: Option<Network>,
}

impl Monitor {
    pub fn new(path: String, iface: Option<String>) -> Monitor {
        Monitor {
            ctx: ComponentContext::new(),
            collect_timer: None,
            cgroups_path: path.clone(),
            memory: Memory::new(path.clone()),
            cpu: Cpu::new(path),
            network: iface.and_then(|i| Some(Network::new(i))),
        }
    }
    fn update(&mut self) {
        match self.memory.update() {
            MemoryStatus::Low => info!(self.ctx.log(), "Current Memory Level: Low"),
            MemoryStatus::Medium=> info!(self.ctx.log(), "Current Memory Level: Medium"),
            MemoryStatus::High => info!(self.ctx.log(), "Current Memory Level: High"),
            MemoryStatus::Critical => info!(self.ctx.log(), "Current Memory Level: Critical!"),
        }
        self.cpu.update();

        if let Some(mut net) = self.network.clone() {
            net.update();
            info!(self.ctx.log(), "Network: {:?}", net);
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
        if let ControlEvent::Start = event {
            let timeout = Duration::from_millis(2000);
            let timer = self.schedule_periodic(timeout, timeout, |self_c, _| {
                 self_c.actor_ref().tell(Box::new(Collect{}), self_c);
             });

            self.collect_timer = Some(timer);
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
        error!(self.ctx.log(), "Got unexpected message from {}", sender);
    }
}