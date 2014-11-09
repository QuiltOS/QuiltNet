use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use Table;
use packet::TcpPacket;
use super::{
  Connection,

  Closed,
  SynSent,
  SynReceived,
  Established,
};

pub trait State {
  fn next<A>(self, &::State<A>, TcpPacket) -> Connection
    where A: RoutingTable;
}

pub fn trans<A>(e: &mut Connection, s: &::State<A>, p: TcpPacket)
  where A: RoutingTable
{
  use std::mem::{uninitialized, swap};

  let mut blank: Connection = unsafe { uninitialized() };

  swap(e, &mut blank);

  *e = match blank {
    Closed         => Closed,
    SynSent    (c) => c.next(s, p),
    SynReceived(c) => c.next(s, p),
    Established(c) => c.next(s, p),
  }
}
