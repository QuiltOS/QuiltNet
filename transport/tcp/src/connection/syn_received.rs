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


pub type RWHandler =
  Box<MyFn<SynReceived, Connection> + Send + Sync + 'static>;

pub struct SynReceived {
  can_read:  RWHandler,
  can_write: RWHandler,
}

impl State for SynReceived
{
  fn next<A>(self,
             _state:  &::State<A>,
             _packet: TcpPacket)
             -> Connection
    where A: RoutingTable
  {
    // stay established
    super::SynReceived(self)
  }
}

impl SynReceived
{
  pub fn passive_new<A>(state:     &::State<A>,
                        us:        ::ConAddr, // already expects specific port
                        them:      ::ConAddr,
                        can_read:  RWHandler,
                        can_write: RWHandler)
                        -> send::Result<()>
  {
    let mut lock0 = state.tcp.read();
    let per_port = access::get_per_port(&mut lock0, us.1)
      .ok().expect("packet should not reach listener if listener doesn't exist");

    let mut lock1 = per_port.connections.write();
    let conn = access::reserve_connection_mut(&mut lock1, them);

    //lock.downgrade(); // TODO: get us a read lock instead
    let mut lock = conn.write();

    match *lock {
      super::Closed => (),
      _ => panic!("packet should never reach listener if connection exists"),
    };

    debug!("2/3 handshake with {} on our port {}", them, us);
    Ok(())
  }
}
