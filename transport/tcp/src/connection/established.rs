use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::io::net::ip::Port;
use std::sync::RWLock;

use misc::interface::{Fn, /* Handler */};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use packet::TcpPacket;
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
  // This is the one bit of information not kept tracked of by our key
  our_addr: ipv4::Addr,
  handler: Handler,
  tcb: TCB,
}

impl super::State for Established
{
  fn next<A>(self,
             _state:  &::State<A>,
             _packet: TcpPacket)
             -> Connection
    where A: RoutingTable
  {

    // stay established
    Connection::Established(self)
  }
}

impl Established
{
  pub fn invoke_handler(mut self, situ: Situation) -> Connection
  {
    use std::mem::{uninitialized, swap};

    let mut handler: Handler = unsafe { uninitialized() };

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

  pub fn new(//state:   &::State<A>,
    us:      ::ConAddr,
    them:    ::ConAddr,
    handler: Handler)
    -> Connection
  {
    debug!("Established connection on our addr {} to server {}", us, them);
    let est = Established {
      our_addr: us.0,
      handler: handler,
      //TODO: initialize TCB with seq number state from handshake
      tcb: TCB::new()
    };
    // first CanRead let's them know connection was made
    est.invoke_handler(Situation::CanRead)
  }
}
