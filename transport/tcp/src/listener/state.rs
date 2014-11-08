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
  fn next<A>(self, &ipv4::State<A>, &Table, TcpPacket) -> Listener
    where A: RoutingTable;
}

pub fn trans<A>(e: &mut Listener, i: &ipv4::State<A>, t: &Table, p: TcpPacket)
  where A: RoutingTable
{
  *e = match e {
    &Closed(ref s) => s.next(i, t, p),
    &Listen(ref s) => s.next(i, t, p),
  }
}
