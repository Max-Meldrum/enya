extern crate pnet;

use pnet::datalink::{self, NetworkInterface};

pub fn find_interface(iface_type: &str) -> Option<NetworkInterface> {
    let interface_match =
        |iface: &NetworkInterface| iface.name.starts_with(iface_type);

    datalink::interfaces()
        .into_iter()
        .filter(interface_match)
        .next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_test() {
        // As interface may depend on host, let's check that it works for "lo"
        let interface = find_interface("lo");
        assert_eq!(interface.is_some(), true);
        assert_eq!(interface.unwrap().is_loopback(), true);
    }
}
