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
use super::syn_received::{
  mod,
  SynReceived,
};
use super::established::{
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
    match self.next_raii(state, packet)
    {
      Ok(con) => con,
      Err(_)  => Connection::Closed,
    }
  }
}



impl SynSent
{
  fn next_raii<A>(self,
                  state:  &::State<A>,
                  packet: TcpPacket)
                  -> send::Result<Connection>
    where A: RoutingTable
  {
    let us   = (packet.get_dst_addr(), packet.get_dst_port());
    let them = (packet.get_src_addr(), packet.get_src_port());

    let normal_trans = if
      packet.flags().contains(packet::ACK | packet::SYN)
    {
      debug!("Done 2/3 handshake with {} on {}", them, us);
      true
    }
    else if
      packet.flags().contains(packet::SYN) &&
      ! packet.flags().contains(packet::ACK)
    {
      debug!("Double Connection on {} from {}? All the way!", us.1, them);
      false
    }
    else
    {
      debug!("Listener on {} got non- syn+ack packet from {}. Why do you let me down server...",
             us.1, them);
      return Err(Error::BadHandshake);
    };

    try!(handshake_3(state, us, them));
    
    Ok(if normal_trans
       {
         // Become established
         Established::new(us,
                          them,
                          self.future_handler)
       }
       else
       {
         // go to SynReceived to wait for their handshake 3
         SynReceived::raw_new(packet.get_seq_num(),
                              self.future_handler)
       })
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
    try!(handshake_1(state, us, them));
    *lock = Connection::SynSent(SynSent { future_handler: handler });
    Ok(())
  }
}



fn handshake_1<A>(state:   &::State<A>,
                  us:      Port, // already expects specific port
                  them:    ::ConAddr)
                  -> send::Result<()>
  where A: RoutingTable
{
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
  // message was sent;
  debug!("Attempt 1/3 handshake with {} on our port {}", them, us);
  Ok(())
}




fn handshake_3<A>(state:   &::State<A>,
                  us:      ::ConAddr,
                  them:    ::ConAddr)
                  -> send::Result<()>
  where A: RoutingTable
{
  let builder: for<'p> |&'p mut packet::TcpPacket| -> send::Result<()> = |packet| {
    use packet::SYN;
    *packet.flags_mut() = SYN;
    Ok(())
  };

  /*
  We do handshake 3 either way
  During double connect, we handshake 3, ACK'ing an immaginary handshake 2
  But the other side will do the same thing, so we get their immaginary ACK
   */
  try!(send::send(&*state.ip,
                  Some(us.0),
                  us.1,
                  them,
                  Some(0),
                  |x| x,
                  builder));

  debug!("Attempt 3/3 handshake with {} on {}", them, us);
  Ok(())
}
