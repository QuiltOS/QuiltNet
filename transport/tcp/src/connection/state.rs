use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use Table;
use packet::TcpPacket;
use super::{
  Connection,

  Closed,
};

pub trait State {
  fn next<A>(self, &ipv4::State<A>, TcpPacket) -> Connection
    where A: RoutingTable;
}

pub fn trans<A>(e: &mut Connection, i: &ipv4::State<A>, p: TcpPacket)
  where A: RoutingTable
{
  *e = match e {
    &Closed(ref s) => s.next(i, p),
  }
}
