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

pub struct RWHandlerPair {
  /// to be called whenever a message arrives
  on_receive:  RWHandler,
  /// to be called whenever the free space in
  /// the outgoing buffer _becomes_ non-empty
  can_send:    RWHandler,
}

pub struct Established {
  // This is the one bit of information not kept tracked of by our key
  our_addr: ipv4::Addr,
  handlers: RWHandlerPair,
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

pub fn new(//state:     &::State<A>,
           us:        ::ConAddr,
           them:      ::ConAddr,
           handlers:  RWHandlerPair)
           -> Established
{
  debug!("Established connection on our addr {} to server {}", us, them);
  Established {
    our_addr: us.0,
    handlers: handlers,
  }
}
