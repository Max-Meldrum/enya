// Traffic Control Tower

use bytes::Buf;
use kompact::*;

#[derive(ComponentDefinition)]
pub struct ControlTower {
    ctx: ComponentContext<ControlTower>,
}

impl ControlTower {
    pub fn new() -> ControlTower {
        ControlTower {
            ctx: ComponentContext::new(),
        }
    }
}

impl Provide<ControlPort> for ControlTower {
    fn handle(&mut self, event: ControlEvent) {
        if let ControlEvent::Start = event {
            info!(self.ctx.log(), "Starting ControlTower");
        }
    }
}

impl Actor for ControlTower {
    fn receive_local(&mut self, _sender: ActorRef, msg: Box<Any>) {}

    fn receive_message(
        &mut self,
        sender: ActorPath,
        _ser_id: u64,
        _buf: &mut Buf,
    ) {
        error!(self.ctx.log(), "Got unexpected message from {}", sender);
    }
}
