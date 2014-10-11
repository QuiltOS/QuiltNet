use std::collections::HashMap;
use std::io::net::ip::IpAddr;
use std::sync::{Arc, RWLock};

use network::ipv4::{
    InterfaceRow,
    IpState,
};
use network::ipv4::strategy::RoutingTable;

#[deriving(Clone)]
pub struct StaticRow {
    pub next_hop:  IpAddr,      // Which link-layer interface to use
}

pub struct StaticTable {
    // key: Ip we want to reach, NOT our interface's IP
    map: RWLock<HashMap<IpAddr, StaticRow>>,
}

impl RoutingTable for StaticTable {

    fn lookup(&self, ip: IpAddr) -> Option<IpAddr> {
        self.map.read().find(&ip).map(|table| table.next_hop)

    }
    
    fn init(elements: &[InterfaceRow]) -> StaticTable {
        // don't need
        let routes_iter = elements
            .iter()
            .map( | &(_, dst, _) | (dst.clone(), StaticRow { next_hop: dst }));
        StaticTable { map: RWLock::new(FromIterator::from_iter(routes_iter)) }
    }

    fn monitor(_state: Arc<IpState<StaticTable>>) -> () {
        // nop
    }
    
    fn dump(&self) {
        for vip in self.map.read().keys() {
            let StaticRow { next_hop } = self.map.read().deref()[*vip];
            println!("{} -> {}", vip, next_hop);
        }
    }

}
