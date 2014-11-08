use network::ipv4::{
  mod,
  send,
  strategy,
};

use packet;
use connection::Connection;

#[deriving(PartialEq, Eq, Clone, Show)]
pub enum Error {
  External(send::Error),
}

pub type Result<T> = ::std::result::Result<T, self::Error>;


pub fn send
  <'ip, 'tcp, A>
  (ip_state:           &'ip ipv4::State<A>,
   connection:         &'tcp Connection,
   dst:                super::ConAddr,
   expected_body_size: Option<u16>,
   builder:            <'a> |&'a mut packet::TcpPacket| -> self::Result<()>)
   -> self::Result<()>
  where A: strategy::RoutingTable
{
  send::send(ip_state,
             dst.0,
             super::PROTOCOL,
             Some(packet::TCP_HDR_LEN as u16 + expected_body_size
                  .unwrap_or(0)),
             | packet | {
               Ok(())
             },
             | packet | {
               Ok(())
             }).map_err(|x| External(x))
}
