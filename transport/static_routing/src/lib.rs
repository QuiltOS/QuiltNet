//#![feature(unboxed_closures)]
//#![feature(slicing_syntax)]
#![feature(phase)]

// for tests
//#![feature(globs)]

#[cfg(not(ndebug))]
#[phase(plugin, link)]
extern crate log;

#[phase(plugin, link)]
extern crate misc;

extern crate network;


use std::collections::HashMap;
use std::io::net::ip::IpAddr;
use std::sync::{Arc, RWLock};

use network::ipv4::IpState;
use network::ipv4::strategy::RoutingTable;

pub struct StaticTable {
  // key:   Ip we want to reach, NOT our interface's IP
  // value: Ip of neighbor we want to send to
  map: RWLock<HashMap<IpAddr, IpAddr>>,
}

impl RoutingTable for StaticTable {

  fn lookup(&self, ip: IpAddr) -> Option<IpAddr> {
    self.map.read().find(&ip).map(|x| *x)
  }
  
  fn init<I>(elements: I) -> StaticTable where I: Iterator<IpAddr> {
    // make I <-> I, the ID map
    let routes_iter = elements.map(|neighbor_ip| (neighbor_ip, neighbor_ip));
    StaticTable { map: RWLock::new(FromIterator::from_iter(routes_iter)) }
  }

  fn monitor(_state: Arc<IpState<StaticTable>>) -> () {
    debug!("Static Routing: In use");
  }

  fn dump(&self) {
    for vip in self.map.read().keys() {
      let next_hop = self.map.read().deref()[*vip];
      println!("{} -> {}", vip, next_hop);
    }
  }

}
