use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::io::net::ip::Port;
use std::sync::RWLock;

use misc::interface::{Fn, /* Handler */};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use packet::TcpPacket;
use super::Connection;
use super::state::State;

pub enum Situation {
  Received,
  CanSend,
}

pub type Handler =
  Box<FnMut<(Established, Situation), Connection> + Send + Sync + 'static>;

pub struct Established {
  // This is the one bit of information not kept tracked of by our key
  our_addr: ipv4::Addr,
  handler: Handler,
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

pub fn new(//state:   &::State<A>,
           us:      ::ConAddr,
           them:    ::ConAddr,
           handler: Handler)
           -> Established
{
  debug!("Established connection on our addr {} to server {}", us, them);
  Established {
    our_addr: us.0,
    handler: handler,
  }
}
