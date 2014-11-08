use Table;
use packet::TcpPacket;
use super::{
  Listener,

  Closed,
  Listen,
};

pub trait State {
  fn next(self, &Table, TcpPacket) -> Listener;
}

pub fn trans(e: &mut Listener, t: &Table, p: TcpPacket) {
  *e = match e {
    &Closed(ref s) => s.next(t, p),
    &Listen(ref s) => s.next(t, p),
  }
}
