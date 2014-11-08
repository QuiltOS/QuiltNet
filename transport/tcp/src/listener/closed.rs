use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use Table;
use packet::TcpPacket;
use super::Listener;
use super::state::State;

pub struct Closed;

impl State for Closed
{
  fn next<A>(self,
             state:   &::State<A>,
             _packet: TcpPacket)
             -> Listener
    where A: RoutingTable
  {
    // stay closed
    super::Closed(Closed)
  }
}
