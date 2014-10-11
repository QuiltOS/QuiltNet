use std::collections::HashMap;
use std::io::net::ip::IpAddr;
use std::sync::{Arc, RWLock};
use std::time::Duration;

use time::{Timespec, get_time};

use network::ipv4::{
    InterfaceRow,
    IpState,
};
use network::ipv4::strategy::RoutingTable;

use transport::static_routing::StaticRow;

mod comm;
mod periodic;
mod packet;

#[deriving(Clone)]
pub struct RipRow {
    pub time_added:   Timespec,  // Relative to 1970
    pub rest:         StaticRow, // just the interface IP address for now
    pub cost:         u8,        // How many hops
    pub learned_from: IpAddr,    // Who we learned this route from (used in split-horizon)
}

pub struct RipTable {
    // key: Ip we want to reach, NOT our interface's IP
    map: RWLock<HashMap<IpAddr, RipRow>>,
}

impl RoutingTable for RipTable {

    fn lookup(&self, ip: IpAddr) -> Option<IpAddr> {
        self.map.read().find(&ip).and_then( |table| {
            Some(table.rest.next_hop)
        })
    }

    fn init(elements: &[InterfaceRow]) -> RipTable {
        let cur_time = get_time();
        // don't need
        let routes_iter = elements.iter()
            .map(|&(ref src, dst, _)|
                 // src is our interface IP, seems like a fine IP to use for the learned-from field
                 (dst.clone(), RipRow {
                     time_added: cur_time,
                     rest: StaticRow { next_hop: dst },
                     cost: 1,
                     learned_from: *src
                 }));
        RipTable { map: RWLock::new(FromIterator::from_iter(routes_iter)) }
    }

    fn monitor(state: Arc<IpState<RipTable>>) -> () {
        comm::register(state.clone());
        periodic::spawn_updater(state.clone());
        periodic::spawn_garbage_collector(state);
    }

    fn dump(&self) {
        for vip in self.map.read().keys() {
            let RipRow { cost, rest, learned_from, time_added } = self.map.read().deref()[*vip];
            println!("{} - {} -> {} [learned from: {}, at: {} ]",
                     vip, cost, rest.next_hop, learned_from, time_added);
        }
    }

}
