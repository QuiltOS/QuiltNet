use std::result;
use std::error::FromError;

use network::ipv4::{
  mod,
  send,
  strategy,
};

use packet;

#[deriving(PartialEq, Eq, Clone, Show)]
pub enum Error {
  ConnectionAlreadyExists,
  ListenerAlreadyExists,
  RouteBrokeConnection, // new route has different src IP so we are fucked
  External(send::Error),
}

pub type Result<T> = ::std::result::Result<T, self::Error>;

impl FromError<ipv4::send::Error> for Error {
  fn from_error(e: ipv4::send::Error) -> Error {
    External(e)
  }
}

pub fn send
  <'ip, 'clos, A, E>
  (ip_state:           &'ip ipv4::State<A>,
   //connection:         &'tcp Connection,
   src:                super::ConAddr,
   dst:                super::ConAddr,
   expected_body_size: Option<u16>,
   builder:            <'a> |&'a mut packet::TcpPacket|:'clos -> result::Result<(), E>,
   upcaster:           |self::Error| -> E)
   -> result::Result<(), E>
  where A: strategy::RoutingTable,
        E: FromError<send::Error>, // + FromError<self::Error>,
{
  let tcp_builder: <'p> |&'p mut ipv4::packet::V| -> result::Result<(), E> = | packet |
  {
    let packet = packet::TcpPacket::hack_mut(packet);

    packet.set_src_port(src.1);
    packet.set_dst_port(dst.1);

    builder(packet)
  };

  let awkward_checksum_fixer: <'p> |&'p mut ipv4::packet::V| -> result::Result<(), E> = | packet |
  {
    let packet = packet::TcpPacket::hack_mut(packet);

    if packet.get_src_addr() != src.0 {
      return Err(upcaster(RouteBrokeConnection))
    };

    packet.update_checksum();
    Ok(())
  };

  try!(send::send::<A, E>(
    ip_state,
    dst.0,
    super::PROTOCOL,
    Some(packet::TCP_HDR_LEN as u16 + expected_body_size.unwrap_or(0)),
    tcp_builder,
    awkward_checksum_fixer));

  Ok(())
}
