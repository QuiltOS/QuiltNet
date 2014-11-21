use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::default::Default;
use std::io::net::ip::Port;
use std::sync::{
  RWLock,
  RWLockReadGuard,
  RWLockWriteGuard,
};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

pub mod state;
pub mod tcb;

pub mod syn_sent;
pub mod syn_received;
pub mod established;

pub enum Connection {
  Closed,
  SynSent(syn_sent::SynSent),
  SynReceived(syn_received::SynReceived),
  Established(established::Established),
}

impl Default for Connection
{
  fn default() -> Connection {
    Connection::Closed
  }
}
