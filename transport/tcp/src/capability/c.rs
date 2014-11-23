use std::io::net::ip::Port;
use std::sync::{Arc, Mutex};
use std::comm::{
  Sender,
  Receiver,
};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use connection::{
  mod,
  Connection,
};
use listener;
use send;

use super::{l, L};


/// Capability that gives synchronous access to a Connection
// TODO: not too great that ipv4::State's routing strategy is leaking this far.
pub struct C<A>
  where A: RoutingTable
{
  us:        Port, // TODO: change to ::ConAddr,
  them:      ::ConAddr,
  //con:       Weak<::connection::Connection>,

  state:     Arc<::State<A>>,

  can_read:  Receiver<()>,
  can_write: Receiver<()>,
}

impl<A> C<A>
  where A: RoutingTable
{
  pub fn connect(state:   &Arc<::State<A>>,
                 us:      Port,
                 them:    ::ConAddr)
                 -> send::Result<C<A>>
  {
    let (handler, rd_rx, wt_rx) = make_con_handler();

    try!(connection::handshaking::Handshaking::new(&**state,
                                                   us,
                                                   them,

                                                   false,
                                                   false,
                                                   None,
                                                   None,
                                                   handler));

    Ok(new(state, us, them, rd_rx, wt_rx))
  }
}

pub fn new<A>(state:   &Arc<::State<A>>,
              us:      Port,
              them:    ::ConAddr,
              rd_rx:   Receiver<()>,
              wt_rx:   Receiver<()>)
              -> C<A>
  where A: RoutingTable
{
  // block on first CanRead---to signify that connection is established
  rd_rx.recv();

  C { us: us,
      them: them,

      state: state.clone(),

      can_read:  rd_rx,
      can_write: wt_rx,
  }
}

pub fn make_con_handler() -> (connection::established::Handler, Receiver<()>, Receiver<()>)
{
  use connection::established::Established;
  use connection::established::Situation;


  let (rd_tx, rd_rx) = channel::<()>();
  let (wt_tx, wt_rx) = channel::<()>();

  let handler = {
    // TODO: this mutex is not necessary
    let rd = Mutex::new(rd_tx);
    let wt = Mutex::new(wt_tx);
    box move |&mut: est: Established, situ: Situation| {
      match situ {
        Situation::CanRead  => rd.lock().send(()),
        Situation::CanWrite => wt.lock().send(()),
      };
      Connection::Established(est)
    }
  };
  (handler, rd_rx, wt_rx)
}
