use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::io::net::ip::Port;
use std::sync::RWLock;

use misc::interface::{MyFn, /* Handler */};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use access;
use packet::TcpPacket;
use send;
use super::Connection;
use super::state::State;

use connection::established::RWHandler;

pub struct SynReceived {
  future_handlers: super::established::RWHandlerPair,
}

impl State for SynReceived
{
  fn next<A>(self,
             _state:  &::State<A>,
             packet: TcpPacket)
             -> Connection
    where A: RoutingTable
  {
    let us   = (packet.get_dst_addr(), packet.get_dst_port());
    let them = (packet.get_src_addr(), packet.get_src_port());

    // Become established
    super::Established(super::established::new(us,
                                               them,
                                               self.future_handlers))
  }
}

pub fn passive_new<A>(state:     &::State<A>,
                      us:        ::ConAddr, // already expects specific port
                      them:      ::ConAddr,
                      handlers:  super::established::RWHandlerPair)
  where A: RoutingTable
{
  let mut lock0 = state.tcp.read();
  let per_port = access::get_per_port(&mut lock0, us.1)
    .ok().expect("Packet should not reach listener if listener doesn't exist");

  let mut lock1 = per_port.connections.write();
  let conn = access::reserve_connection_mut(&mut lock1, them);

  //lock.downgrade(); // TODO: get us a read lock instead
  let mut lock = conn.write();

  *lock = match *lock {
    super::Closed => super::SynReceived(SynReceived { future_handlers: handlers }),
    _ => panic!("Packet should never reach listener if connection exists"),
  };

  debug!("Done with 2/3 handshake with {} on our port {}", them, us.1);
}
