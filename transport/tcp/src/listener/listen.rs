use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::io::net::ip::Port;
use std::sync::RWLock;

use misc::interface::{MyFn, /* Handler */};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use access;
use Table;
use packet;
use packet::TcpPacket;
use super::Listener;
use super::state::State;

use connection::established::RWHandlerPair;

pub type OnConnectionAttempt = Box<MyFn<(::ConAddr /* us */, ::ConAddr /* them */,),
                                         Option<RWHandlerPair>>
                                   + Send + Sync + 'static>;

pub struct Listen {
  handler: OnConnectionAttempt,
}

impl State for Listen
{
  fn next<A>(self,
             state:  &::State<A>,
             packet: TcpPacket)
             -> Listener
    where A: RoutingTable
  {
    let us   = (packet.get_dst_addr(), packet.get_dst_port());
    let them = (packet.get_src_addr(), packet.get_src_port());

    if !( packet.flags().contains(packet::SYN) && ! packet.flags().contains(packet::ACK) )
    {
      debug!("Listener on {} got non-syn or ack packet from {}. This is not how you make an introduction....",
             us.1, them);
      return super::Listen(self); // TODO: Macro to make early return less annoying
    };

    if packet.get_payload().len() != 0 {
      debug!("Listener on {} got non-empty packet from {}. Slow down, we just met....", us.1, them);
      return super::Listen(self);
    };

    debug!("Done with 1/3 handshake with {} on our port {}", them, us.1);

    let handler_pair = match self.handler.call((us, them)) {
      Some(hs) => hs,
      None     => return super::Listen(self),
    };
    ::connection::syn_received::passive_new(state, us, them, handler_pair);
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

    *lock = match *lock {
      super::Closed => super::Listen(Listen { handler: handler }),
      _             => return Err(()),
    };

    Ok(())
  }
}
