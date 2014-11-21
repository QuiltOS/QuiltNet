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

pub struct SynSent {
  future_handler: established::Handler,
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
      return Connection::Closed; // TODO: Macro to make early return less annoying
    };
    debug!("Done 2/3 handshake with {} on {}", them, us);


    let builder: for<'p> |&'p mut packet::TcpPacket| -> send::Result<()> = |packet| {
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
      Err(_) => return Connection::Closed, // TODO: Call conn failure continuation
    };

    debug!("Attempt 3/3 handshake with {} on {}", them, us);

    // Become established
        let est = established::new(us,
                               them,
                                   self.future_handler);
    // first CanRead lets them know connection was made
    est.invoke_handler(Situation::CanRead)
  }
}

pub fn active_new<A>(state:   &::State<A>,
                     us:      Port,
                     them:    ::ConAddr,
                     handler: established::Handler)
                     -> send::Result<()>
  where A: RoutingTable
{
  let per_port = ::PerPort::get_or_init(&state.tcp, us);
  let conn = per_port.connections.get_or_init(them,
                                              || RWLock::new(Default::default()));

  let mut lock = conn.write();
  match *lock {
    Connection::Closed => (),
    _                  => return Err(Error::ConnectionAlreadyExists),
  };

  debug!("Confirmed no existing connection on our port {} to server {}", us, them);

  let builder: for<'p> |&'p mut packet::TcpPacket| -> send::Result<()> = |packet| {
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
  *lock = Connection::SynSent(SynSent { future_handler: handler });

  debug!("Attempt 1/3 handshake with {} on our port {}", them, us);
  Ok(())
}
