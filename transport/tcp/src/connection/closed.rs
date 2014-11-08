use packet::TcpPacket;
use super::Connection;
use super::state::State;

pub struct Closed;

impl State for Closed
{
  fn next(self, _p: TcpPacket) -> Connection {
    super::Closed(Closed)
  }
}
