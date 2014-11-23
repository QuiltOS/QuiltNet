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


type FugginComplex = (::ConAddr, ::ConAddr, Sender<connection::established::Handler>);

/// Capability that gives synchronous access to a Listener
// TODO: not too great that ipv4::State's routing strategy is leaking this far.
pub struct L<A>
  where A: RoutingTable
{
  us:       Port,
  weak:     Weak<::PerPort>,

  state:    Arc<::State<A>>,
  requests: Receiver<FugginComplex>
}

impl<A> L<A>
  where A: RoutingTable
{
  pub fn listen(state:      &Arc<::State<A>>,
                us:         Port)
                -> send::Result<L<A>>
  {
    let (tx, rx) = channel::<FugginComplex>();

    let handler = {
      // TODO: this mutex is not necessary
      let request = Mutex::new(tx);
      box move |&mut: us: ::ConAddr, them: ::ConAddr| {
        debug!("in L-Capability Handler");
        let (tx, rx) = channel();
        request.lock().send((us, them, tx));
        Some(rx.recv())
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

  pub fn accept(&self) -> C<A> {
    let (us, them, reply) = self.requests.recv();

    let (handler, rd_rx, wt_rx) = c::make_con_handler();

    // We will initialize these, then the async side will just find a closed
    // connection -- no problem.
    //
    // async does not reserve connection before calling us in case we decline to
    // accept
    let per_port = ::PerPort::get_or_init(&self.state.tcp, us.1);
    let conn     = Connection::get_or_init(&*per_port, them);

    // give them the stuff to make a connection
    reply.send(handler);

    c::new(&self.state, conn.downgrade(), rd_rx, wt_rx)
  }
}
