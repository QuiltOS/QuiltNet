use std::io::net::ip::Port;
use std::sync::{Arc, Mutex};
use std::comm::{
  Sender,
  Receiver,
};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use connection;
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


/*
/// Capability that gives synchronous access to a Listener
struct L<A>
  where A: RoutingTable
{
  local_port:       Port,

  state:          Arc<super::State<A>>,

  pub candidates: Receiver<C<A>>,
}

struct AcceptorFn(pub Sender<C<A>>);

impl self::L
{
  pub fn new(state:      Arc<super::State<A>>,
             local_port: Port) -> self::L
  {
    let (tx, rx) = channel::<C<A>>();
    self::L {
      local_port: local_port,
      state: state,
      canidates: rx,
    }
  }
}
*/

/// Capability that gives synchronous access to a Connection
struct C<A>
  where A: RoutingTable
{
  us:        Port, // TODO: change to ::ConAddr,
  them:      ::ConAddr,

  state:     Arc<super::State<A>>,
  
  can_read:  Receiver<()>,
  can_write: Receiver<()>,
}

impl<A> C<A>
  where A: RoutingTable
{
  pub fn connect(state:   Arc<super::State<A>>,
                 us:      Port,
                 them:    ::ConAddr)
                 -> send::Result<C<A>>
  {
    use connection::established::Established;
    use connection::established::{Situation, CanRead, CanWrite};
    
    let (rd_tx, rd_rx) = channel::<()>();
    let (wt_tx, wt_rx) = channel::<()>();
    
    let handler = {
      // TODO: this mutex is not necessary 
      let rd = Mutex::new(rd_tx);
      let wt = Mutex::new(wt_tx);
      box move |&mut : est: Established, situ: Situation| {
        match situ {
          CanRead  => rd.lock().send(()),
          CanWrite => wt.lock().send(()),
        };
        connection::Established(est)
      }
    };
    
    try!(connection::syn_sent::active_new(&*state, us, them, handler));

    // block on first CanRead---to signify that connection is established
    rd_rx.recv();
    
    Ok(C { us: us,
           them: them,

           state: state,

           can_read:  rd_rx,
           can_write: wt_rx,
    })
  }
}
