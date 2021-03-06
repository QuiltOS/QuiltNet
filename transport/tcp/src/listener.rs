use std::fmt;
use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::io::net::ip::Port;
use std::sync::{Arc, Weak, RWLock};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use Table;
use packet::{mod, TcpPacket};
use send::{mod, Error,};

use connection::Connection;
use connection::handshaking::Handshaking;
use connection::established::{
  mod,
  Established,
};

pub type ConnectionFun = Box<
  FnOnce<(established::Handler,),
         (send::Result<Weak<RWLock<Connection>>>)>
  + Send + Sync + 'static>;

pub type ConnectionAttemptMessage = (::ConAddr, // us
                                     ::ConAddr, // them
                                     ConnectionFun);

pub type OnConnectionAttempt = Box<
  FnMut<ConnectionAttemptMessage, bool>
  + Send + Sync + 'static>;


pub struct Listener {
  us:      Port,
  handler: OnConnectionAttempt,
}

impl Listener
{
  fn handle<A>(&mut self,
               state:  &Arc<::State<A>>,
               packet: TcpPacket)
               -> bool
    where A: RoutingTable
  {
    let us   = (packet.get_dst_addr(), packet.get_dst_port());
    let them = (packet.get_src_addr(), packet.get_src_port());

    assert_eq!(self.us, us.1);

    if !( packet.flags().contains(packet::SYN) && ! packet.flags().contains(packet::ACK) )
    {
      debug!("Listener on {} got non-syn or ack packet from {}. This is not how you make an introduction....",
             us.1, them);
      return true;
    };

    if packet.get_payload().len() != 0 {
      debug!("Listener on {} got non-empty packet from {}. Slow down, we just met....", us.1, them);
      return true;
    };

    debug!("Done with 1/3 handshake with {} on our port {}", them, us.1);

    let con_maker: ConnectionFun = {
      let seq_num = packet.get_seq_num();
      let state   = state.clone().downgrade();
      let wnd     = packet.get_window_size();
      box move |: handler | {
        let state = match state.upgrade() {
          None    => return Err(Error::BadHandshake), // I suppose IP dissapearing is a bad handshake?
          Some(s) => s,
        };
        Handshaking::new(&state, us.1, Some(us.0), them,
                         false, true, Some(seq_num), Some(wnd), handler)
      }
    };
    self.handler.call_mut((us, them, con_maker))
  }
}

pub fn trans<A>(listener: &mut Option<Listener>,
                state:    &Arc<::State<A>>,
                packet:   TcpPacket)
  where A: RoutingTable
{
  use std::mem::swap;

  let mut blank: Option<Listener>= None;

  swap(listener, &mut blank);

  *listener = match blank {
    None    => {
      debug!("Sorry, no l to receive this packet");
      None
    },
    Some(mut l) => {
      debug!("Listener found!");
      if l.handle(state, packet) {
        Some(l)
      } else {
        None
      }
    }
  }
}


pub fn passive_new<A>(state:      &::State<A>,
                      handler:    OnConnectionAttempt,
                      local_port: Port)
                      -> send::Result<Weak<::PerPort>>
  where A: RoutingTable
{
  let per_port = ::PerPort::get_or_init(&state.tcp, local_port);
  let mut lock = per_port.listener.write(); // get listener read lock

  *lock = match *lock {
    None => Some(Listener { us: local_port, handler: handler }),
    _    => {
      debug!("Oh no, listener already exists here");
      return Err(Error::PortOrTripleReserved);
    },
  };

  Ok(per_port.clone().downgrade())
}

impl fmt::Show for Listener {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "LISTENER: <{}>", self.us)
  }
}
