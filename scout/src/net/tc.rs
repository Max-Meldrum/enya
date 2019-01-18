// Linux Traffic Control Bindings

pub enum Qdisc {
    Prio,
    Htb,
}

pub struct Tc {}

impl Tc {
    pub fn new() -> Tc {
        Tc {}
    }

    pub fn create_qdisc(qdisc: Qdisc) {
        match qdisc {
            Qdisc::Prio => (),
            Qdisc::Htb => (),
        }
    }

    fn exec() {}
}
