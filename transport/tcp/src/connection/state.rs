use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use Table;
use packet::TcpPacket;
use super::{
  Connection,

  Closed,
};

pub trait State {
  fn next<A>(self, &::State<A>, TcpPacket) -> Connection
    where A: RoutingTable;
}

pub fn trans<A>(e: &mut Connection, s: &::State<A>, p: TcpPacket)
  where A: RoutingTable
{
  *e = match e {
    &Closed(ref c) => c.next(s, p),
  }
}
