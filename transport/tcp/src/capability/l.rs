use std::io::net::ip::Port;
use std::sync::{Arc, Weak, Mutex};
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

use super::{c, C};


/// Capability that gives synchronous access to a Listener
// TODO: not too great that ipv4::State's routing strategy is leaking this far.
pub struct L<A>
  where A: RoutingTable
{
  us:       Port,
  weak:     Weak<::PerPort>,

  state:    Arc<::State<A>>,
  requests: Receiver<listener::ConnectionAttemptMessage>
}

impl<A> L<A>
  where A: RoutingTable
{
  pub fn listen(state:      &Arc<::State<A>>,
                us:         Port)
                -> send::Result<L<A>>
  {
    let (tx, rx) = channel::<listener::ConnectionAttemptMessage>();

    let handler = {
      // TODO: this mutex is not necessary
      let request = Mutex::new(tx);
      box move |&mut: us: ::ConAddr, them: ::ConAddr, call | {
        debug!("in L-Capability Handler");
        request.lock().send((us, them, call));
        true // keep on listening
      }
    };

    let weak = try!(::listener::passive_new::<A>(
      &**state,
      handler,
      us));

    Ok(self::L {
      us:       us,
      weak:     weak,

      state:    state.clone(),
      requests: rx,
    })
  }

  pub fn accept(&self) -> send::Result<C<A>> {
    debug!("Accept called on capability {}", self.us);
    // get info and black-box function to actually make the connection
    let (_, _, mk_con) = self.requests.recv();
    // make handler and receives to interact with connection
    let (handler, rd_rx, wt_rx) = c::make_con_handler();
    // make connection with handler
    let weak_ref = try!(mk_con.call_once((handler,)));
    // make connection capability with weak ref to handler
    Ok(c::new(&self.state, weak_ref, rd_rx, wt_rx))
  }
}
