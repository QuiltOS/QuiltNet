use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::io::net::ip::Port;
use std::sync::RWLock;

use misc::interface::{Fn, /* Handler */};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use access;
use packet;
use packet::TcpPacket;
use send;
use super::Connection;
use super::state::State;

use connection::established::RWHandler;

pub struct SynSent {
  future_handlers: super::established::RWHandlerPair,
}

impl State for SynSent
{
  fn next<A>(self,
             state:  &::State<A>,
             packet: TcpPacket)
             -> Connection
    where A: RoutingTable
  {
    let us   = (packet.get_dst_addr(), packet.get_dst_port());
    let them = (packet.get_src_addr(), packet.get_src_port());

    if !packet.flags().contains(packet::ACK | packet::SYN)
    {
      debug!("Listener on {} got non- syn+ack packet from {}. Why do you let me down server...",
             us.1, them);
      return super::Closed; // TODO: Macro to make early return less annoying
    };
    debug!("Done 2/3 handshake with {} on {}", them, us);


    let builder: <'p> |&'p mut packet::TcpPacket| -> send::Result<()> = |packet| {
      use packet::SYN;
      *packet.flags_mut() = SYN;
      Ok(())
    };

    match send::send(&*state.ip,
                     Some(us.0),
                     us.1,
                     them,
                     Some(0),
                     |x| x,
                     builder)
    {
      Ok(_)  => (),
      Err(_) => return super::Closed, // TODO: Call conn failure continuation
    };

    debug!("Attempt 3/3 handshake with {} on {}", them, us);

    // Become established
    super::Established(super::established::new(us,
                                               them,
                                               self.future_handlers))
  }
}

pub fn active_new<A>(state:     &::State<A>,
                     us:        Port,
                     them:      ::ConAddr,
                     handlers:  super::established::RWHandlerPair)
                     -> send::Result<()>
  where A: RoutingTable
{
  let mut lock0 = state.tcp.write();
  let per_port = access::reserve_per_port_mut(&mut lock0, us);

  let mut lock1 = per_port.connections.write();
  let conn = access::reserve_connection_mut(&mut lock1, them);

  //lock.downgrade(); // TODO: get us a read lock instead
  let mut lock = conn.write();

  match *lock {
    super::Closed => (),
    _             => return Err(send::ConnectionAlreadyExists),
  };

  debug!("Confirmed no existing connection on our port {} to server {}", us, them);

  let builder: <'p> |&'p mut packet::TcpPacket| -> send::Result<()> = |packet| {
    use packet::SYN;
    *packet.flags_mut() = SYN;
    Ok(())
  };

  try!(send::send(&*state.ip,
                  None,
                  us,
                  them,
                  Some(0),
                  |x| x,
                  builder));

  // don't bother really reserving port until at least the first
  // message was sent
  *lock = super::SynSent(SynSent { future_handlers: handlers });

  debug!("Attempt 1/3 handshake with {} on our port {}", them, us);
  Ok(())
}
