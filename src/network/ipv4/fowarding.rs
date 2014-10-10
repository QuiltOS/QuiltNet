use std::collections::HashMap;
use std::io::net::ip::IpAddr;
use std::sync::{Arc, RWLock};

use super::{
    InterfaceRow,
};

use super::strategy::RoutingTable;

pub struct FowardingRow {
    pub next_hop:  IpAddr,      // Which link-layer interface to use
}

pub struct FowardingTable {
    // key: Ip we want to reach, NOT our interface's IP
    map: RWLock<HashMap<IpAddr, FowardingRow>>,
}

impl RoutingTable for FowardingTable {

    fn lookup(&self, ip: IpAddr) -> Option<IpAddr> {
        self.map.read().find(&ip).map(| &FowardingRow { ref next_hop, ..} |
                                     *next_hop)

    }
    
    fn init(elements: &[InterfaceRow]) -> FowardingTable {
        // don't need
        let routes_iter = elements.iter()
            .map( | &(_, dst, _) |
                      // src is our interface IP, seems like a fine IP to use for the learned-from field
                      (dst.clone(), FowardingRow { next_hop: dst }));
        FowardingTable { map: RWLock::new(FromIterator::from_iter(routes_iter)) }
    }

    fn dump(&self) {
        for vip in self.map.read().keys() {
            let FowardingRow { next_hop } = self.map.read().deref()[*vip];
            println!("{} -> {}", vip, next_hop);
        }
    }

}
