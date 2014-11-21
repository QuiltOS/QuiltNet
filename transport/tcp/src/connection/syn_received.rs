use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::default::Default;
use std::io::net::ip::Port;
use std::sync::RWLock;

use misc::interface::{Fn, /* Handler */};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use packet::{mod, TcpPacket};
use send::{mod, Error,};
use super::Connection;
use super::state::State;

use connection::established::{
  mod,
  Established,
  Situation,
};

pub struct SynReceived {
  future_handler: established::Handler,
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
      return Connection::Closed; // TODO: Macro to make early return less annoying
    };
    debug!("Done 3/3 handshake with {} on {}", them, us);

    // Become established
    let est = established::new(us,
                               them,
                               self.future_handler);
    // first CanRead let's them know connection was made
    est.invoke_handler(Situation::CanRead)
  }
}

impl SynReceived
{

}

pub fn passive_new<A>(state:   &::State<A>,
                      us:      ::ConAddr, // already expects specific port
                      them:    ::ConAddr,
                      handler: established::Handler)
  where A: RoutingTable
{
  let per_port = ::PerPort::get_or_init(&state.tcp, us.1);
  let conn = per_port.connections.get_or_init(them,
                                              || RWLock::new(Default::default()));

  let mut lock = conn.write();
  match *lock {
    Connection::Closed => (),
    _ => panic!("Packet should never reach listener if connection exists"),
  };

  // TODO: Report ICE if this signature is removed
  let builder: for<'p> |&'p mut packet::TcpPacket| -> send::Result<()> = |packet| {
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

  *lock = Connection::SynReceived(SynReceived { future_handler: handler });

  debug!("Attempt 2/3 handshake with {} on our port {}", them, us.1);
}
