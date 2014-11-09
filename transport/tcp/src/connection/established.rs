use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::io::net::ip::Port;
use std::sync::RWLock;

use misc::interface::{MyFn, /* Handler */};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use packet::TcpPacket;
use super::Connection;
use super::state::State;


pub type RWHandler =
  Box<MyFn<Established, Connection> + Send + Sync + 'static>;

pub struct Established {
  can_read:  RWHandler,
  can_write: RWHandler,
}

impl State for Established
{
  fn next<A>(self,
             _state:  &::State<A>,
             _packet: TcpPacket)
             -> Connection
    where A: RoutingTable
  {
    // stay established
    super::Established(self)
  }
}

impl Established
{

}
