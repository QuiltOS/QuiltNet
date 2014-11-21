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


/// A TCP Capability is mostly analogous to a (TCP) Socket on
/// Unix. This library wraps the more low-level core implementation to
/// provide a "blocking", synchronous interface.
///
/// While in the core interface connections and listeners need to be
/// explicitly closed, that would be undesirable burden on the users
/// of this implementation. Once all capabilities are dropped, nobody
/// will be able to access the underlying connection without using the
/// core interface. Therefore, once they are all dropped the
/// connection will be automatically closed. If you don't like that,
/// use the core interface.

// TODO: not too great that ipv4::State's routing strategy is leaking this far.
// TODO: clonable capabilities






type FugginComplex = (::ConAddr, ::ConAddr, Sender<connection::established::Handler>);

/// Capability that gives synchronous access to a Listener
pub struct L<A>
  where A: RoutingTable
{
  us:       Port,
  state:    Arc<super::State<A>>,
  requests: Receiver<FugginComplex>
}

impl<A> L<A>
  where A: RoutingTable
{
  pub fn new(state:      &Arc<super::State<A>>,
             us:         Port)
             -> send::Result<L<A>>
  {
    let (tx, rx) = channel::<FugginComplex>();

    let handler = {
      // TODO: this mutex is not necessary
      let request = Mutex::new(tx);
      box move |&mut: us: ::ConAddr, them: ::ConAddr| {
        let (tx, rx) = channel();
        request.lock().send((us, them, tx));
        Some(rx.recv())
      }
    };

    Ok(self::L {
      us: us,
      state: state.clone(),
      requests: rx,
    })
  }

  pub fn accept(self) -> C<A> {
    let (us, them, reply) = self.requests.recv();

    let (handler, rd_rx, wt_rx) = make_con_handler();

    // give them the stuff to make a connection
    reply.send(handler);

    // block on first CanRead---to signify that connection is established
    rd_rx.recv();

    C { us: us.1,
        them: them,

        state: self.state.clone(),

        can_read:  rd_rx,
        can_write: wt_rx,
    }
  }
}







/// Capability that gives synchronous access to a Connection
pub struct C<A>
  where A: RoutingTable
{
  us:        Port, // TODO: change to ::ConAddr,
  them:      ::ConAddr,

  state:     Arc<super::State<A>>,

  can_read:  Receiver<()>,
  can_write: Receiver<()>,
}

fn make_con_handler() -> (connection::established::Handler, Receiver<()>, Receiver<()>)
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

impl<A> C<A>
  where A: RoutingTable
{
  pub fn connect(state:   &Arc<super::State<A>>,
                 us:      Port,
                 them:    ::ConAddr)
                 -> send::Result<C<A>>
  {
    let (handler, rd_rx, wt_rx) = make_con_handler();

    try!(connection::syn_sent::active_new(&**state, us, them, handler));

    // block on first CanRead---to signify that connection is established
    rd_rx.recv();

    Ok(C { us: us,
           them: them,

           state: state.clone(),

           can_read:  rd_rx,
           can_write: wt_rx,
    })
  }
}
