use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use packet::TcpPacket;
use super::Connection;
use super::state::State;

pub struct Closed;

impl State for Closed
{
  fn next<A>(self,
             _ip_state: &ipv4::State<A>,
             _packet: TcpPacket)
             -> Connection
    where A: RoutingTable
  {
    // stay closed
    super::Closed(Closed)
  }
}
