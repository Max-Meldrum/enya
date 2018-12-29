extern crate pnet;

use pnet::datalink::{self, NetworkInterface};

pub fn find_interfaces(iface_type: &str) -> Vec<NetworkInterface> {
    let interface_match =
        |iface: &NetworkInterface| iface.name.starts_with(iface_type);

    datalink::interfaces()
        .into_iter()
        .filter(interface_match)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_test() {
        // As interface may depend on host, let's check that it works for "lo"
        let ifaces = find_interfaces("lo");
        assert_eq!(ifaces.len(), 1);
        assert_eq!(ifaces.first().unwrap().is_loopback(), true);
    }

}
