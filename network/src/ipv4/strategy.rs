use std::io::net::ip::{Ipv4Addr, IpAddr};
use std::option::Option;
use std::sync::Arc;

use super::IpState;

pub trait RoutingTable: Send + Sync {

  // initialized with the neighbor IPs
  fn init<I>(i: I) -> Self where I: Iterator<IpAddr>;

  fn lookup(&self, IpAddr) -> Option<IpAddr>;

  fn monitor(state: Arc<IpState<Self>>) -> ();

  fn dump(&self);

}

pub fn init_hack<RT, I>(i: I) -> RT
  where RT: RoutingTable, I: Iterator<IpAddr>
{
  RoutingTable::init::<I>(i)
}

pub fn monitor_hack<RT>(s: Arc<IpState<RT>>) where RT: RoutingTable
{
  RoutingTable::monitor(s);
}
