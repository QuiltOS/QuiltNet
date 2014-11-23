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
use super::established::{
  mod,
  Established,
  Situation,
};

pub struct Handshaking {
  want:           bool, // do we want to we want to receive an ACK?
  owe:            bool, // ought we to send them an ACK, if the situation arises?
  our_number:     u32,
  their_number:   Option<u32>,
  our_ip:         Option<ipv4::Addr>,
  future_handler: established::Handler,
}

impl super::State for Handshaking
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



impl Handshaking
{
  fn next_raii<A>(mut self,
                  state:  &::State<A>,
                  packet: TcpPacket)
                  -> send::Result<Connection>
    where A: RoutingTable
  {
    let us   = (packet.get_dst_addr(), packet.get_dst_port());
    let them = (packet.get_src_addr(), packet.get_src_port());

    debug!("{} to {} pre packet: want {}, owe {}", them, us, self.want, self.owe);
    
    if packet.flags().contains(packet::ACK) {
      self.want = false;
    }

    if packet.flags().contains(packet::SYN) {
      self.owe = true
    }

    debug!("{} to {} post packet: want {}, owe {}", them, us, self.want, self.owe);
    
    try!(self.send(state, us.1, them, false /* don't SYN out of blue */));

    Ok(if (self.want || self.owe) == false {
      debug!("{} to {} free!!!!", them, us);
      // Become established
      Established::new(us,
                       them,
                       self.future_handler)
    } else {
      debug!("{} to {} not free", them, us);
      Connection::Handshaking(self)
    })
  }

  pub fn new<A>(state:          &::State<A>,
                us:             Port,
                them:           ::ConAddr,

                want:           bool,
                owe:            bool,
                their_number:   Option<u32>,
                our_ip:         Option<ipv4::Addr>,
                future_handler: established::Handler)
                       -> send::Result<()>
    where A: RoutingTable
  {
    let per_port = ::PerPort::get_or_init(&state.tcp, us);
    let conn = per_port.connections.get_or_init(them,
                                                || RWLock::new(Default::default()));
    let mut lock = conn.write();
    match *lock {
      Connection::Closed => (),
      _                  => return Err(Error::PortOrTripleReserved),
    };
    debug!("Confirmed no existing connection on our port {} to server {}", us, them);

    let mut potential = Handshaking {
      want:           want,
      owe:            owe,
      our_number:     0, // TODO make random
      their_number:   their_number,
      our_ip:         our_ip,
      future_handler: future_handler,
    };

    try!(potential.send(state, us, them, true /* initial SYN */));

    // don't bother really reserving port until at least the first
    // message was sent;
    *lock = super::Connection::Handshaking(potential);
    Ok(())
  }



  fn send<A>(&mut self,
             state:   &::State<A>,
             us:      Port, // already expects specific port
             them:    ::ConAddr,
             brag:    bool)
             -> send::Result<()>
    where A: RoutingTable
  {
    let our_ip = self.our_ip;

    debug!("{} to {} pre send: want {}, owe {}", them, us, self.want, self.owe);

    {
      let builder: for<'p> |&'p mut packet::TcpPacket| -> send::Result<()> = |packet|
      {   
        if brag {
          debug!("{} will SYN {}", us, them);
          packet.flags_mut().insert(packet::SYN);
          self.want = true;
        } 
        if self.owe {
          debug!("{} will ACK {}", us, them);
          packet.flags_mut().insert(packet::ACK);
          self.owe = false;
        }
        Ok(())
      };

      try!(send::send(&*state.ip,
                      our_ip, // to ensure routes don't fuck us
                      us,
                      them,
                      Some(0),
                      |x| x,
                      builder));
    }

    debug!("{} to {} post send: want {}, owe {}", them, us, self.want, self.owe);
    
    debug!("Attempt handshake with {} on our port {}", them, us);
    Ok(())
  }
}
