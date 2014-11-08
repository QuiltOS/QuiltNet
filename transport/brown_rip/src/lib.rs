//#![feature(unboxed_closures)]
//#![feature(slicing_syntax)]
#![feature(phase)]

// for tests
#![feature(globs)]

#[cfg(not(ndebug))]
#[phase(plugin, link)]
extern crate log;

#[phase(plugin, link)]
extern crate misc;
extern crate time;
extern crate network;

use std::collections::HashMap;
use std::sync::{Arc, RWLock};

use time::{Timespec, get_time};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

mod comm;
mod periodic;
mod packet;

const RIP_INFINITY:    u8  = 16;
const RIP_MAX_ENTRIES: u16 = 64;
const RIP_PROTOCOL:    u8  = 200;

#[deriving(Clone)]
pub struct RipRow {
  // the next hop is always the same node that told your about the route 
  pub next_hop:     ipv4::Addr,    // which neighbor to we send the packet too
  pub time_added:   Timespec,  // Relative to 1970
  pub cost:         u8,        // How many hops
}

pub struct RipTable {
  // key: ipv4:: we want to reach, NOT our interface's IP
  map: RWLock<HashMap<ipv4::Addr, RipRow>>,
}

impl RoutingTable for RipTable {

  fn lookup(&self, ip: ipv4::Addr) -> Option<ipv4::Addr> {
    self.map.read().get(&ip).and_then( |table| {
      Some(table.next_hop)
    })
  }

  fn init<I>(elements: I) -> RipTable where I: Iterator<ipv4::Addr> {
    let cur_time = get_time();
    // don't need
    let routes_iter = elements.map(
      |neighbor_addr|
      // src is our interface IP, seems like a fine IP to use for the learned-from field
      (neighbor_addr, RipRow {
        time_added: cur_time,
        next_hop: neighbor_addr,
        cost: 1,
      }));
    RipTable { map: RWLock::new(FromIterator::from_iter(routes_iter)) }
  }

  fn monitor(state: Arc<ipv4::State<RipTable>>) -> () {
    debug!("In use");
    comm::register(state.clone());
    periodic::spawn_updater(state.clone());
    periodic::spawn_garbage_collector(state);
  }

  fn dump(&self) {
    for dst in self.map.read().keys() {
      let RipRow { cost, next_hop, time_added } = self.map.read().deref()[*dst];
      println!("{} - {} -> {} [learned at: {} ]",
               dst, cost, next_hop, time_added);
    }
  }

}
