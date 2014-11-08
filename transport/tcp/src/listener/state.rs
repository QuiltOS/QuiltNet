use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use Table;
use packet::TcpPacket;
use super::{
  Listener,

  Closed,
  Listen,
};

pub trait State {
  fn next<A>(self, &::State<A>, TcpPacket) -> Listener
    where A: RoutingTable;
}

pub fn trans<A>(e: &mut Listener,
                s: &::State<A>,
                p: TcpPacket)
  where A: RoutingTable
{
  use std::mem::{uninitialized, swap};

  let mut blank: Listener = unsafe { uninitialized() };

  swap(e, &mut blank);

  *e = match blank {
    Closed    => Closed,
    Listen(l) => l.next(s, p),
  }
}
