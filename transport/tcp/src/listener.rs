use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::io::net::ip::Port;
use std::sync::{Weak, RWLock};

use misc::interface::{Fn, /* Handler */};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use Table;
use packet::{mod, TcpPacket};
use send::{mod, Error,};

use connection::handshaking::Handshaking;
use connection::established::{
  mod,
  Established,
};

pub type OnConnectionAttempt = Box<FnMut<(::ConAddr /* us */, ::ConAddr /* them */,),
                                         Option<established::Handler>>
                               + Send + Sync + 'static>;

pub struct Listener {
  us:      Port,
  handler: OnConnectionAttempt,
}

impl Listener
{
  fn handle<A>(&mut self,
               state:  &::State<A>,
               packet: TcpPacket)
    where A: RoutingTable
  {
    let us   = (packet.get_dst_addr(), packet.get_dst_port());
    let them = (packet.get_src_addr(), packet.get_src_port());

    assert_eq!(self.us, us.1);

    if !( packet.flags().contains(packet::SYN) && ! packet.flags().contains(packet::ACK) )
    {
      debug!("Listener on {} got non-syn or ack packet from {}. This is not how you make an introduction....",
             us.1, them);
      return;
    };

    if packet.get_payload().len() != 0 {
      debug!("Listener on {} got non-empty packet from {}. Slow down, we just met....", us.1, them);
      return;
    };

    debug!("Done with 1/3 handshake with {} on our port {}", them, us.1);

    let handler_pair = match self.handler.call_mut((us, them)) {
      Some(hs) => hs,
      None     => return,
    };
    let _ = Handshaking::new(state,
                             us.1,
                             Some(us.0),
                             them,

                             false,
                             true,
                             Some(packet.get_seq_num()),
                             handler_pair);
    // keep on listening
  }
}

pub fn trans<A>(listener: &mut Option<Listener>,
                state:    &::State<A>,
                packet:   TcpPacket)
  where A: RoutingTable
{
  match listener {
    &None    => debug!("Sorry, no listener to receive this packet"),
    &Some(ref mut l) => {
      debug!("Listener found!");
      l.handle(state, packet)
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
