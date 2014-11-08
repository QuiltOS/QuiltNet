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
use std::sync::{Arc, RWLock};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

pub struct StaticTable {
  // key:   Ip we want to reach, NOT our interface's IP
  // value: Ip of neighbor we want to send to
  map: RWLock<HashMap<ipv4::Addr, ipv4::Addr>>,
}

impl RoutingTable for StaticTable {

  fn lookup(&self, ip: ipv4::Addr) -> Option<ipv4::Addr> {
    self.map.read().get(&ip).map(|x| *x)
  }
  
  fn init<I>(elements: I) -> StaticTable where I: Iterator<ipv4::Addr> {
    // make I <-> I, the ID map
    let routes_iter = elements.map(|neighbor_ip| (neighbor_ip, neighbor_ip));
    StaticTable { map: RWLock::new(FromIterator::from_iter(routes_iter)) }
  }

  fn monitor(_state: Arc<ipv4::State<StaticTable>>) -> () {
    debug!("In use");
  }

  fn dump(&self) {
    for vip in self.map.read().keys() {
      let next_hop = self.map.read().deref()[*vip];
      println!("{} -> {}", vip, next_hop);
    }
  }

}
