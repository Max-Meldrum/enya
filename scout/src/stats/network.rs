use crate::util;
use std::cell::Cell;

#[derive(Debug, Clone)]
pub struct Network {
    veth: String,
    pub rx_bytes: Cell<u64>,
    pub rx_packets: Cell<u64>,
    pub tx_bytes: Cell<u64>,
    pub tx_packets: Cell<u64>,
}

impl Network {
    pub fn new(iface: String) -> Network {
        Network {
            veth: iface,
            rx_bytes: Cell::new(0),
            rx_packets: Cell::new(0),
            tx_bytes: Cell::new(0),
            tx_packets: Cell::new(0),
        }
    }

    pub fn update(&mut self) {
        let rx_bytes_path = Network::path(self, "rx_bytes");
        let rx_bytes = util::read_u64_from(&rx_bytes_path);

        let rx_packets_path = Network::path(self, "rx_packets");
        let rx_packets = util::read_u64_from(&rx_packets_path);

        let tx_bytes_path = Network::path(self, "tx_bytes");
        let tx_bytes = util::read_u64_from(&tx_bytes_path);

        let tx_packets_path = Network::path(self, "tx_packets");
        let tx_packets = util::read_u64_from(&tx_packets_path);

        self.rx_bytes.set(rx_bytes.unwrap_or(0));
        self.rx_packets.set(rx_packets.unwrap_or(0));

        self.tx_bytes.set(tx_bytes.unwrap_or(0));
        self.tx_packets.set(tx_packets.unwrap_or(0));
    }

    fn path(&mut self, file_name: &str) -> String {
        "sys/class/net".to_owned()
            + &self.veth.to_owned()
            + "/statistics/"
            + file_name
    }
}
