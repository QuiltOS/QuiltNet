#[macro_use]
extern crate log;

#[macro_use]
extern crate misc;

extern crate network;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

#[derive(Debug)]
pub struct StaticTable {
  // key:   Ip we want to reach, NOT our interface's IP
  // value: Ip of neighbor we want to send to
  map: RwLock<HashMap<ipv4::Addr, ipv4::Addr>>,
}

impl<'a> RoutingTable<'a> for StaticTable {

  fn lookup(&self, ip: ipv4::Addr) -> Option<ipv4::Addr> {
    self.map.read().unwrap().get(&ip).map(|x| *x)
  }

  fn init<I>(elements: I) -> StaticTable where I: Iterator<Item=ipv4::Addr> {
    // make I <-> I, the ID map
    let routes_iter = elements.map(|neighbor_ip| (neighbor_ip, neighbor_ip));
    StaticTable { map: RwLock::new(routes_iter.collect()) }
  }

  fn monitor<E>(_state: Arc<ipv4::State<'a, StaticTable, E>>) -> () {
    debug!("In use");
  }

  fn dump(&self) {
    for vip in self.map.read().unwrap().keys() {
      let next_hop = self.map.read().unwrap()[vip];
      info!("{} -> {}", vip, next_hop);
    }
  }

}
