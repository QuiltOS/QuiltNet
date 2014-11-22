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
  init_seq_num:   u32,
  future_handler: established::Handler,
}

impl State for SynReceived
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



impl SynReceived
{
  fn next_raii<A>(self,
                  _state:  &::State<A>,
                  packet: TcpPacket)
                  -> send::Result<Connection>
    where A: RoutingTable
  {
    let us   = (packet.get_dst_addr(), packet.get_dst_port());
    let them = (packet.get_src_addr(), packet.get_src_port());

    if !packet.flags().contains(packet::ACK)
    {
      debug!("Listener on {} got non-ack packet from {}. Make a friendship just to wreck it client?",
             us.1, them);
      return Err(Error::BadHandshake)
    };
    debug!("Done 3/3 handshake with {} on {}", them, us);

    // Become established
    Ok(Established::new(us,
                        them,
                        self.future_handler))
  }



  pub fn passive_new<A>(state:       &::State<A>,
                        us:          ::ConAddr, // already expects specific port
                        them:        ::ConAddr,
                        init_seq_no: u32,
                        handler:     established::Handler)
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

    match handshake_2(state, us, them) {
      Err(_)  => (),
      Ok(con) => *lock = SynReceived::raw_new(init_seq_no, handler),
    };
  }

  /// Used to implement double connect, otherwise use `passive_new`
  pub fn raw_new(init_seq_no: u32,
                 handler:     established::Handler)
                 -> Connection
  {
    Connection::SynReceived(SynReceived {
      init_seq_num:   init_seq_no,
      future_handler: handler,
    })
  }
}



fn handshake_2<A>(state:   &::State<A>,
                  us:      ::ConAddr, // already expects specific port
                  them:    ::ConAddr)
                  -> send::Result<()>
  where A: RoutingTable
{
  // TODO: Report ICE if this signature is removed
  let builder: for<'p> |&'p mut packet::TcpPacket| -> send::Result<()> = |packet| {
    use packet::{SYN, ACK};
    *packet.flags_mut() = SYN | ACK;
    Ok(())
  };

  // TODO: Should we keep track of a failure to respond?
  try!(send::send(&*state.ip,
                  Some(us.0),
                  us.1,
                  them,
                  Some(0),
                  |x| x,
                  builder));

  debug!("Attempt 2/3 handshake with {} on our port {}", them, us.1);
  Ok(())
}
