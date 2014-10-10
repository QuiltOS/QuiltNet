use std::collections::HashMap;
use std::io::net::ip::IpAddr;
use std::sync::{Arc, RWLock};

use network::ipv4::InterfaceRow;
use network::ipv4::strategy::RoutingTable;

use transport::static_routing::ForwardingRow;

pub struct RipRow {
    pub rest: ForwardingRow,
    pub cost:      u8,          // How many hops
    pub learned_from: IpAddr,   // Who we learned this route from (used in split-horizon)
}

pub struct RipTable {
    // key: Ip we want to reach, NOT our interface's IP
    map: RWLock<HashMap<IpAddr, RipRow>>,
}

impl RoutingTable for RipTable {

    fn lookup(&self, ip: IpAddr) -> Option<IpAddr> {
        self.map.read().find(&ip).map(|table| table.rest.next_hop)

    }
    
    fn init(elements: &[InterfaceRow]) -> RipTable {
        // don't need
        let routes_iter = elements.iter()
            .map( | &(ref src, dst, _) |
                      // src is our interface IP, seems like a fine IP to use for the learned-from field
                      (dst.clone(), RipRow { rest: ForwardingRow { next_hop: dst }, cost: 1, learned_from: *src }));
        RipTable { map: RWLock::new(FromIterator::from_iter(routes_iter)) }
    }

    fn dump(&self) {
        for vip in self.map.read().keys() {
            let RipRow {cost, rest, learned_from} = self.map.read().deref()[*vip];
            println!("{} - {} -> {} [learned from: {}]", vip, cost, rest.next_hop, learned_from);
        }
    }

}
