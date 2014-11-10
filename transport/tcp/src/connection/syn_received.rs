use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::io::net::ip::Port;
use std::sync::RWLock;

use misc::interface::{MyFn, /* Handler */};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use access;
use packet;
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

    if !packet.flags().contains(packet::ACK)
    {
      debug!("Listener on {} got non-ack packet from {}. Make a friendship just to wreck it client?",
             us.1, them);
      return super::Closed; // TODO: Macro to make early return less annoying
    };
    debug!("Done 3/3 handshake with {} on {}", them, us);

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

  match *lock {
    super::Closed => (),
    _ => panic!("Packet should never reach listener if connection exists"),
  };

  // TODO: Report ICE if this signature is removed
  let builder: <'p> |&'p mut packet::TcpPacket| -> send::Result<()> = |packet| {
    use packet::{SYN, ACK};
    *packet.flags_mut() = SYN | ACK;
    Ok(())
  };

  // TODO: Should we keep track of a failure to respond?
  match send::send(&*state.ip,
                   Some(us.0),
                   us.1,
                   them,
                   Some(0),
                   |x| x,
                   builder)
  {
    Ok(_)  => (),
    Err(_) => return,
  };

  *lock = super::SynReceived(SynReceived { future_handlers: handlers });

  debug!("Attempt 2/3 handshake with {} on our port {}", them, us.1);
}
