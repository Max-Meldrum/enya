use crate::util;

#[derive(Debug, Clone)]
pub struct Network {
    interface: String,
    tx_bytes_path: String,
    tx_packets_path: String,
    rx_bytes_path: String,
    rx_packets_path: String,
    pub rx_bytes: u64,
    pub rx_packets: u64,
    pub tx_bytes: u64,
    pub tx_packets: u64,
}

impl Network {
    pub fn new(iface: String) -> Network {
        let tx_b = "/sys/class/net/".to_owned()
            + &iface.to_owned()
            + "/statistics/tx_bytes";

        let tx_p = "/sys/class/net/".to_owned()
            + &iface.to_owned()
            + "/statistics/tx_packets";

        let rx_b = "/sys/class/net/".to_owned()
            + &iface.to_owned()
            + "/statistics/rx_bytes";

        let rx_p = "/sys/class/net/".to_owned()
            + &iface.to_owned()
            + "/statistics/rx_packets";

        Network {
            interface: iface,
            tx_bytes_path: tx_b,
            tx_packets_path: tx_p,
            rx_bytes_path: rx_b,    
            rx_packets_path: rx_p,
            rx_bytes: 0,
            rx_packets: 0,
            tx_bytes: 0,
            tx_packets: 0,
        }
    }

    pub fn update(&mut self) {
        self.tx_bytes = util::read_u64_from(&self.tx_bytes_path)
            .unwrap_or(0);

        self.tx_packets = util::read_u64_from(&self.tx_packets_path)
            .unwrap_or(0);

        self.rx_bytes = util::read_u64_from(&self.rx_bytes_path)
            .unwrap_or(0);

        self.rx_packets = util::read_u64_from(&self.rx_packets_path)
            .unwrap_or(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_test() {
        let mut network = Network::new(String::from("lo"));
        network.update();
        // Might be the case that this is actually zero
        assert!(network.rx_bytes > 0);
    }
}
