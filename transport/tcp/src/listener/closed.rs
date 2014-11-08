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
             _ip_state: &ipv4::State<A>,
             _tcp_state: &Table,
             _packet: TcpPacket)
             -> Listener
    where A: RoutingTable
  {
    super::Listen(super::listen::Listen)
  }
}
