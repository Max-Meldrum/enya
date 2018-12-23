extern crate iptables;

// TODO create functionality for TC + IpTables

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let ipt = iptables::new(false).unwrap();
        assert_eq!(2 + 2, 4);
    }
}
