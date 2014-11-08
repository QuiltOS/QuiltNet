use std::io::net::ip::Port;
use std::sync::Arc;
use std::comm::{
  Sender,
  Receiver,
};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

/*

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
      candidates: rx,
    }
  }
}

struct ReadFn {

}

struct WriteFn {

}

/// Capability that gives synchronous access to a Connection
struct C<A>
  where A: RoutingTable
{
  src:       ConAddr,
  dst:       ConAddr,

  tcp_state: Arc<super::Table>,
  ip_state:  Arc<ipv4::State<A>>,

  can_read:  Receiver<()>,
  can_write: Receiver<()>,
}
*/
