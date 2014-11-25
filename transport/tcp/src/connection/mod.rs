use std::fmt;

use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::default::Default;
use std::io::net::ip::Port;
use std::sync::{
  Arc,
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

impl Connection {
  pub fn get_or_init(per_port: &::PerPort, them: ::ConAddr) -> Arc<RWLock<Connection>>
  {
    per_port.connections.get_or_init(them,
                                     || RWLock::new(Default::default()))
  }
}

impl fmt::Show for Connection {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
       &Connection::Closed => write!(f, "CLOSED"),
       &Connection::Handshaking(ref h) => write!(f, "HANDSHAKING[TODO]"),
       &Connection::Established(ref e) => write!(f, "ESTABLISHED"),
    }
  }
}

pub trait State {
  fn next<A>(self, &::State<A>, TcpPacket) -> Connection
    where A: RoutingTable;
}

pub fn trans<A>(e: &mut Connection, s: &::State<A>, p: TcpPacket)
  where A: RoutingTable
{
  use std::mem::swap;

  let mut blank: Connection = Connection::Closed;

  // safe to "close" it without another connection moving in because we have lock
  swap(e, &mut blank);

  *e = match blank {
    Connection::Closed         => Connection::Closed,
    Connection::Handshaking(c) => c.next(s, p),
    Connection::Established(c) => c.next(s, p),
  }
}
