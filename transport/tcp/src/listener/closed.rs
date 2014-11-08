use Table;
use packet::TcpPacket;
use super::Listener;
use super::state::State;

pub struct Closed;

impl State for Closed
{
  fn next(self, _t: &Table, _p: TcpPacket) -> Listener {
    super::Listen(super::listen::Listen)
  }
}
