use std::fmt;

use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::default::Default;
use std::io::net::ip::Port;
use std::io::Timer;
use std::sync::{
  Arc,
  RWLock,
  RWLockReadGuard,
  RWLockWriteGuard,
};
use std::time::duration::Duration;

use network::ipv4;
use network::ipv4::strategy::RoutingTable;
use Table;
use packet::TcpPacket;

pub mod packet_buf;
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

  fn close<A>(self, &::State<A>) -> Connection
    where A: RoutingTable;

  fn checkup<A>(self, &::State<A>, &mut Duration) -> (Connection, bool)
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

pub fn close<A>(e: &mut Connection, s: &::State<A>) where A: RoutingTable
{
  use std::mem::swap;

  let mut blank: Connection = Connection::Closed;

  // safe to "close" it without another connection moving in because we have lock
  swap(e, &mut blank);

  *e = match blank {
    Connection::Closed         => Connection::Closed,
    Connection::Handshaking(c) => c.close(s),
    Connection::Established(c) => c.close(s),
  }
}

pub fn checkup<A>(e: &mut Connection,
                  s: &::State<A>,
                  interval: &mut Duration)
                  -> bool
  where A: RoutingTable
{
  use std::mem::swap;

  let mut blank: Connection = Connection::Closed;

  // safe to "close" it without another connection moving in because we have lock
  swap(e, &mut blank);

  let (new_con, stop) = match blank {
    Connection::Closed         => (Connection::Closed, true),
    Connection::Handshaking(c) => c.checkup(s, interval),
    Connection::Established(c) => c.checkup(s, interval),
  };
  *e = new_con;

  stop
}
