use network::ipv4::{
  mod,
  send,
  strategy,
};

use packet;

#[deriving(PartialEq, Eq, Clone, Show)]
pub enum Error {
  External(send::Error),
}

pub type Result<T> = ::std::result::Result<T, self::Error>;


pub fn send
  <'ip, 'tcp, A>
  (ip_state:           &'ip ipv4::State<A>,
   tcp_state:          &'tcp super::Table,
   dst:                super::ConAddr,
   expected_body_size: Option<u16>,
   builder:            <'a> |&'a mut packet::TcpPacket| -> self::Result<()>)
   -> self::Result<()>
  where A: strategy::RoutingTable
{
  Ok(())
}
