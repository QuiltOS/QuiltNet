use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::io::net::ip::Port;
use std::sync::RWLock;

use misc::interface::{MyFn, /* Handler */};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use access;
use Table;
use packet::TcpPacket;
use super::Listener;
use super::state::State;

use connection::established::RWHandler;

pub type OnConnectionAttempt = //Handler<Ip>;
  // us, them
  Box<MyFn<(Port, ::ConAddr,), Option<[RWHandler, ..2]>> + Send + Sync + 'static>;

pub struct Listen {
  handler: OnConnectionAttempt,
}

impl State for Listen
{
  fn next<A>(self,
             _state:  &::State<A>,
             _packet: TcpPacket)
             -> Listener
    where A: RoutingTable
  {
    // keep on listening
    super::Listen(self)
  }
}

impl Listen
{
  pub fn new<A>(state:      &::State<A>,
                handler:    OnConnectionAttempt,
                local_port: Port)
                -> Result<(), ()>
  {
    let mut lock = state.tcp.write();
    let per_port = access::reserve_per_port_mut(&mut lock, local_port);

    //lock.downgrade(); // TODO: get us a read lock instead
    let mut lock = per_port.listener.write(); // get listener read lock

    match *lock {
      super::Closed => (),
      _             => return Err(()),
    }

    *lock = super::Listen(Listen { handler: handler });
    Ok(())
  }
}
