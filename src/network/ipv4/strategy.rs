use std::io::net::ip::{Ipv4Addr, IpAddr};
use std::option::Option;
use std::sync::Arc;

use packet::ipv4 as packet;

use super::{
  IpState,
  InterfaceRow,
};

pub trait RoutingTable: Send + Sync {
  
  // initialized with the neighbor IPs
  fn init<I>(I) -> Self where I: Iterator<IpAddr>;

  fn lookup(&self, IpAddr) -> Option<IpAddr>;

  fn monitor(state: Arc<IpState<Self>>) -> ();

  fn dump(&self);

}
