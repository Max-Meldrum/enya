extern crate caps;
extern crate iptables;

mod error;
mod interface;
mod tc;

use crate::error::ErrorKind::*;
use crate::error::*;
use crate::tc::*;
use caps::{CapSet, Capability};
use iptables::*;

pub struct EnyaNet {
    ip_tables: IPTables,
    tc: Tc,
}

impl EnyaNet {
    pub fn new() -> Result<EnyaNet> {
        let ip_tables = iptables::new(false)
            .map_err(|e| Error::with_cause(IpTablesError, e))?;

        if !EnyaNet::is_net_admin() {
            Err(Error::new(NetAdminError))
        } else {
            Ok(EnyaNet {
                ip_tables,
                tc: Tc::new(),
            })
        }
    }

    fn is_net_admin() -> bool {
        let net_admin =
            caps::has_cap(None, CapSet::Permitted, Capability::CAP_NET_ADMIN);
        net_admin.unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

}
