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
  *e = match e {
    &Closed(ref l) => l.next(s, p),
    &Listen(ref l) => l.next(s, p),
  }
}
