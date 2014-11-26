use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::io::net::ip::Port;
use std::sync::{Arc,RWLock, Weak};
use std::time::duration::Duration;

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use packet::{mod, TcpPacket};
use send::{mod, Error,};

use super::Connection;
use self::tcb::TCB;
use self::tcb::timer;


pub mod tcb;


pub enum Situation {
  CanRead,
  CanWrite,
}

pub type Handler =
  Box<FnMut<(Established, Situation), Connection> + Send + Sync + 'static>;

// especially must be private because we cheat on whether the fields exist
pub struct Established {
  us:             ::ConAddr,
  them:           ::ConAddr,

  handler: Handler,
  tcb: TCB,
}

impl super::State for Established
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

  fn close<A>(self, _state: &::State<A>) -> Connection
    where A: RoutingTable
  {
    debug!("TODO: close for established");
    Connection::Established(self)
  }

  fn checkup<A>(mut self,
                state: &::State<A>,
                interval: &mut Duration)
                -> (Connection, bool)
    where A: RoutingTable
  {
    debug!("TODO: checkup for established");

    tcb::timer::on_timeout(&mut self, state, interval);
    
    // false == don't kill timer thread
    (Connection::Established(self), false)
  }
}

impl Established
{
  fn next_raii<A>(mut self,
                  state:  &::State<A>,
                  packet: TcpPacket)
                  -> send::Result<Connection>
    where A: RoutingTable
  {
    let us   = (packet.get_dst_addr(), packet.get_dst_port());
    let them = (packet.get_src_addr(), packet.get_src_port());

    assert_eq!(self.us,   us);
    assert_eq!(self.them, them);

    debug!("TCB bout to recv!");
    let (r, w) = self.tcb.recv(packet);
    debug!("TCB recv yielded (r:{}, w:{})", r, w);

    let self2 = if r {
      self.invoke_handler(Situation::CanRead)
    } else {
      Connection::Established(self)
    };

    let self3 = match self2 {
      Connection::Established(est) => if w {
        est.invoke_handler(Situation::CanWrite)
      } else {
        Connection::Established(est)
      },
      _ => self2,
    };


    // TODO Check if control packet -> transition to close

    // TODO
    // Handle as received data:
    //  If enitrely outside window, discard
    //  If we can add to window, do this
    //    If this means application can read more, signal by calling CanRead

    // self.tcb.recv(_packet, CanRead);

    // stay established
    Ok(self3)
  }


  pub fn invoke_handler(mut self, situ: Situation) -> Connection
  {
    use std::mem::swap;

    debug!("Established connection is invoking its handler");
    let mut handler: Handler = box move |&mut: _, _| {
      debug!("I am a dummy closure used by swap, don't call me!!!");
      panic!();
    };

    // 1st swap
    swap(&mut self.handler, &mut handler);

    let mut con: Connection = handler.call_mut((self, situ));

    match con {
      // 2nd swap
      Connection::Established(ref mut e) => swap(&mut e.handler, &mut handler),
      _ => (),
    };

    con
  }

  pub fn new<A>(state:     &::State<A>,
                us:        ::ConAddr,
                them:      ::ConAddr,
                our_isn:   u32,
                their_isn: u32,
                their_wnd: u16,
                handler:   Handler)
                -> Connection
    where A: RoutingTable
  {
    debug!("Established connection on our addr {} to server {}", us, them);
    let est = Established {
      us:      us,
      them:    them,
      handler: handler,
      tcb:     TCB::new(our_isn, their_isn, their_wnd)
    };

    // first CanRead let's them know connection was made
    est.invoke_handler(Situation::CanRead)
  }

  /// non-blocking, returns how much was written to caller's buffer
  pub fn read<A>(&mut self,
                 state:   &::State<A>,
                 buf:     &mut [u8])
                 -> uint
    where A: RoutingTable
  {
    debug!("trying to do a non-blocking read: est");
    self.tcb.read(buf)
  }

  /// non-blocking, returns how much was read from caller's buffer
  pub fn write<A>(&mut self,
                  state:   &::State<A>,
                  buf:     &[u8])
                  -> uint
    where A: RoutingTable
  {
    debug!("trying to do a non-blocking write");
    self.tcb.send(buf, state, self.us, self.them)
  }

  pub fn get_window(&self) -> ((u32, u16), (u32, u16)) {
    self.tcb.get_window()
  }
}
