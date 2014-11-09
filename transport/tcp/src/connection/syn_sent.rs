use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::io::net::ip::Port;
use std::sync::RWLock;

use misc::interface::{MyFn, /* Handler */};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use access;
use packet::TcpPacket;
use super::Connection;
use super::state::State;


pub type RWHandler =
  Box<MyFn<SynSent, Connection> + Send + Sync + 'static>;

pub struct SynSent {
  can_read:  RWHandler,
  can_write: RWHandler,
}

impl State for SynSent
{
  fn next<A>(self,
             _state:  &::State<A>,
             _packet: TcpPacket)
             -> Connection
    where A: RoutingTable
  {
    // stay established
    super::SynSent(self)
  }
}

impl SynSent
{
  pub fn active_new<A>(state:     &::State<A>,
                       us:        Port,
                       them:      ::ConAddr,
                       can_read:  RWHandler,
                       can_write: RWHandler)
                       -> Result<(), ()>
  {
    let mut lock0 = state.tcp.write();
    let per_port = access::reserve_per_port_mut(&mut lock0, us);

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
