use bytes::Buf;
use kompact::Timer;

use kompact::*;
use std::time::Duration;
use std::cell::Cell;


use crate::stats::memory::Memory;



#[derive(Clone, Copy)]
struct Collect {}


#[derive(ComponentDefinition)]
pub struct Monitor {
    ctx: ComponentContext<Monitor>,
    cgroups_path: String,
    memory: Memory,
}

impl Monitor {
    pub fn new(path: String) -> Monitor {
        Monitor {
            ctx: ComponentContext::new(),
            cgroups_path: path,
            memory: Memory::new(),
        }
    }
}

impl Provide<ControlPort> for Monitor {
    fn handle(&mut self, event: ControlEvent) {
        if let ControlEvent::Start = event {
            let timeout = Duration::from_millis(2000);
            self.schedule_periodic(timeout, timeout, |self_c, _| {
                 self_c.actor_ref().tell(Box::new(Collect{}), self_c);
             });
        }
    }
}

impl Actor for Monitor {
    fn receive_local(&mut self, _sender: ActorRef, msg: Box<Any>) {
        if let Ok(collect) = msg.downcast::<Collect>() {
            self.memory.stats(&self.cgroups_path);
            info!(self.ctx.log(), "Memory {:?}", self.memory);
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
