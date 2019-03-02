// Linux Traffic Control Bindings

// Classful QDISCS
pub enum Qdisc {
    Prio, // PRIO qdisc with class 1,2 and 3
}

pub struct Tc {}

impl Tc {
    pub fn new() -> Tc {
        Tc {}
    }

    pub fn create_qdisc(qdisc: Qdisc) {
        match qdisc {
            Qdisc::Prio => (),
        }
    }

    fn exec() {}
}
