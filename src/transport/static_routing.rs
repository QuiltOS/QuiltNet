use std::collections::HashMap;
use std::io::net::ip::IpAddr;
use std::sync::{Arc, RWLock};

use network::ipv4::InterfaceRow;
use network::ipv4::strategy::RoutingTable;

pub struct ForwardingRow {
    pub next_hop:  IpAddr,      // Which link-layer interface to use
}

pub struct ForwardingTable {
    // key: Ip we want to reach, NOT our interface's IP
    map: RWLock<HashMap<IpAddr, ForwardingRow>>,
}

impl RoutingTable for ForwardingTable {

    fn lookup(&self, ip: IpAddr) -> Option<IpAddr> {
        self.map.read().find(&ip).map(| &ForwardingRow { ref next_hop, ..} |
                                     *next_hop)

    }
    
    fn init(elements: &[InterfaceRow]) -> ForwardingTable {
        // don't need
        let routes_iter = elements.iter()
            .map( | &(_, dst, _) |
                      // src is our interface IP, seems like a fine IP to use for the learned-from field
                      (dst.clone(), ForwardingRow { next_hop: dst }));
        ForwardingTable { map: RWLock::new(FromIterator::from_iter(routes_iter)) }
    }

    fn dump(&self) {
        for vip in self.map.read().keys() {
            let ForwardingRow { next_hop } = self.map.read().deref()[*vip];
            println!("{} -> {}", vip, next_hop);
        }
    }

}
