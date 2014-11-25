use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::default::Default;
use std::io::net::ip::Port;
use std::sync::{RWLock, Weak};
use std::rand::{task_rng, Rng};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use packet::{mod, TcpPacket};
use send::{mod, Error,};
use super::tcb::{mod};

use super::Connection;
use super::established::{
  mod,
  Established,
  Situation,
};

pub struct Handshaking {
  us:             Port,
  our_ip:         Option<ipv4::Addr>,
  them:           ::ConAddr,

  want:           bool, // do we want to we want to receive an ACK?
  owe:            bool, // ought we to send them an ACK, if the situation arises?
  ackd_before:    bool,
  synd_before:    bool,

  our_number:     u32,
  their_number:   Option<u32>,
  their_wnd:      Option<u16>,
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

    match self.our_ip {
      Some(ip) => assert_eq!(ip, us.0),
      None     => self.our_ip = Some(us.0),
    };
    assert_eq!(self.them, them);

    debug!("{} to {} pre packet: want {}, owe {}", them, us, self.want, self.owe);

    if packet.flags().contains(packet::ACK) {
      self.want = false;
      //FIXME: should we verify the ACK# accompanying this?
      self.their_wnd = Some(packet.get_window_size());
    }

    if packet.flags().contains(packet::SYN) {
      self.owe = true;
      self.their_number = Some(packet.get_seq_num()); // TODO: should check this
      self.their_wnd = Some(packet.get_window_size());
    }

    debug!("{} to {} post packet: want {}, owe {}", them, us, self.want, self.owe);

    try!(self.send(state, us.1, them));

    Ok(if (self.want || self.owe) == false {
      debug!("{} to {} free!!!!", them, us);
      // Become established
      Established::new((self.our_ip.unwrap(), self.us),
                       self.them,
                       self.our_number,
                       self.their_number.unwrap(),
                       self.their_wnd.unwrap(),
                       self.future_handler)
    } else {
      debug!("{} to {} not free", them, us);
      Connection::Handshaking(self)
    })
  }

  pub fn new<A>(state:          &::State<A>,
                us:             Port,
                our_ip:         Option<ipv4::Addr>,
                them:           ::ConAddr,

                want:           bool,
                owe:            bool,
                their_number:   Option<u32>,
                their_wnd:      Option<u16>,
                future_handler: established::Handler)
                -> send::Result<Weak<RWLock<Connection>>>
    where A: RoutingTable
  {
    let per_port = ::PerPort::get_or_init(&state.tcp, us);
    let conn     = Connection::get_or_init(&*per_port, them);
    {
      let mut lock = conn.write();
      match *lock {
        Connection::Closed => (),
        _                  => return Err(Error::PortOrTripleReserved),
      };
      debug!("Confirmed no existing connection on our port {} to server {}", us, them);

      let mut potential = Handshaking {
        us:             us,
        our_ip:         our_ip,
        them:           them,

        want:           want,
        owe:            owe,
        ackd_before:    false,
        synd_before:    false,
        our_number:     Handshaking::generate_isn(),
        their_number:   their_number,
        their_wnd:      their_wnd,
        future_handler: future_handler,
      };

      try!(potential.send(state, us, them));

      // don't bother really reserving port until at least the first
      // message was sent;
      *lock = super::Connection::Handshaking(potential);
    }
    Ok(conn.downgrade())
  }



  fn send<A>(&mut self,
             state:   &::State<A>,
             us:      Port, // already expects specific port
             them:    ::ConAddr)
             -> send::Result<()>
    where A: RoutingTable
  {
    let our_ip = self.our_ip;

    debug!("{} to {} pre send: want {}, owe {}", them, us, self.want, self.owe);

    {
      let builder: for<'p> |&'p mut packet::TcpPacket| -> send::Result<()> = |packet|
      {
        if ! self.synd_before { // gotta SYN them at least once for double handshake
          debug!("{} will SYN {}", us, them);
          packet.flags_mut().insert(packet::SYN);

          self.want = true;
          self.synd_before = true;
        }
        if self.owe {
          debug!("{} will ACK {}", us, them);

          packet.flags_mut().insert(packet::ACK);
          self.owe = false;
          self.ackd_before = true;
        }

        // Set SEQ to our ISN
        packet.set_seq_num(self.our_number);

        // Set ACK to their SEQ if we need
        match self.their_number {
          None => (),
          Some(ack_num) => packet.set_ack_num(self.their_number.unwrap()),
        };

        // Set Window Size
        packet.set_window_size(tcb::TCP_RECV_WND_INIT);

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


  /// Generates a pseudorandom u32 - used to set ISN
  fn generate_isn() -> u32 {
    let mut rng = task_rng();
    rng.gen::<u32>()
  }

  pub fn close(self) -> Connection
  {
    if self.ackd_before {
      debug!("TODO: goto fin wait 1");
      Connection::Handshaking(self)
    } else { // can close immediately
      Connection::Closed
    }
  }
}
