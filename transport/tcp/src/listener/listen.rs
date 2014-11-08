use Table;
use packet::TcpPacket;
use super::Listener;
use super::state::State;

pub struct Listen;

impl State for Listen
{
  fn next(self, _t: &Table, _p: TcpPacket) -> Listener {
    // keep on listening
    super::Listen(super::listen::Listen)
  }
}
