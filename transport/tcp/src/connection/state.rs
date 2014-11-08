use Table;
use packet::TcpPacket;
use super::{
  Connection,

  Closed,
};

pub trait State {
  fn next(self, TcpPacket) -> Connection;
}

pub fn trans(e: &mut Connection, p: TcpPacket) {
  *e = match e {
    &Closed(ref s) => s.next(p),
  }
}
