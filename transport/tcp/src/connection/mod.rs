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
use Table;
use packet::TcpPacket;

pub mod manager;

pub mod tcb;

pub mod handshaking;
pub mod established;


pub enum Connection {
  Closed,
  Handshaking(handshaking::Handshaking),
  Established(established::Established),
}

impl Default for Connection
{
  fn default() -> Connection {
    Connection::Closed
  }
}


pub trait State {
  fn next<A>(self, &::State<A>, TcpPacket) -> Connection
    where A: RoutingTable;
}

pub fn trans<A>(e: &mut Connection, s: &::State<A>, p: TcpPacket)
  where A: RoutingTable
{
  use std::mem::{uninitialized, swap};

  let mut blank: Connection = unsafe { uninitialized() };

  swap(e, &mut blank);

  *e = match blank {
    Connection::Closed         => Connection::Closed,
    Connection::Handshaking(c) => c.next(s, p),
    Connection::Established(c) => c.next(s, p),
  }
}
