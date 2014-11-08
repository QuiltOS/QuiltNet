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
  pub fn new<A>(state:     &::State<A>,
                us:        Port,
                them:      ::ConAddr,
                can_read:  RWHandler,
                can_write: RWHandler)
                -> Result<(), ()>
  {
    let mut lock = state.tcp.read();

    let per_port = match (*lock).get(&us) {
      None    => panic!("but.. but... we exist!"),
      Some(p) => p,
    };

    let mut lock = per_port.connections.write();

    let conn = match lock.entry(them) {
      Vacant(entry)   => entry.set(RWLock::new(super::Closed)),
      Occupied(entry) => entry.into_mut(),
    };

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
