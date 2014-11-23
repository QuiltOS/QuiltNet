use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::io::net::ip::Port;
use std::sync::RWLock;

use misc::interface::{Fn, /* Handler */};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use packet::{mod, TcpPacket};
use send::{mod, Error,};

use super::Connection;
use super::tcb::TCB;

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

    // TODO Check if control packet -> transition to close

    // TODO
    // Handle as received data:
    //  If enitrely outside window, discard
    //  If we can add to window, do this
    //    If this means application can read more, signal by calling CanRead

    // self.tcb.recv(_packet, CanRead);

    // stay established
    Ok(Connection::Established(self))
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

  pub fn new(//state:     &::State<A>,
             us:        ::ConAddr,
             them:      ::ConAddr,
             our_isn:   u32,
             their_isn: u32,
             handler:   Handler)
    -> Connection
  {
    debug!("Established connection on our addr {} to server {}", us, them);
    let est = Established {
      us:      us,
      them:    them,
      handler: handler,
      //TODO: initialize TCB with seq number state from handshake
      tcb:     TCB::new(our_isn, their_isn)
    };
    // first CanRead let's them know connection was made
    est.invoke_handler(Situation::CanRead)
  }
}
