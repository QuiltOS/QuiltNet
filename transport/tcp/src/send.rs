use std::result;
use std::io::net::ip::Port;
use std::error::FromError;

use network::ipv4::{
  mod,
  send,
  strategy,
};

use packet;

#[deriving(PartialEq, Eq, Clone, Show)]
pub enum Error {
  PortOrTripleReserved,
  ListenerAlreadyExists,
  RouteBrokeConnection, // new route has different src IP so we are fucked
  BadHandshake,
  External(send::Error),
}

pub type Result<T> = ::std::result::Result<T, self::Error>;

impl FromError<ipv4::send::Error> for Error {
  fn from_error(e: ipv4::send::Error) -> Error {
    Error::External(e)
  }
}

pub fn send
  <'ip, 'clos, A, E>
  (ip_state:           &'ip ipv4::State<A>,
   //connection:         &'tcp Connection,
   src_addr:           Option<ipv4::Addr>,
   src_port:           Port,
   dst:                super::ConAddr,
   expected_body_size: Option<u16>,
   upcaster:           |self::Error| -> E,
   builder:            for<'a> |&'a mut packet::TcpPacket|:'clos -> result::Result<(), E>)
   -> result::Result<(), E>
  where A: strategy::RoutingTable,
        E: FromError<send::Error>,
{
  let tcp_builder: for<'p> |&'p mut ipv4::packet::V| -> result::Result<(), E> = | packet |
  {
    // make room for TCP header
    let new_len = packet.as_vec().len() + packet::TCP_HDR_LEN;
    unsafe { packet.as_mut_vec().set_len(new_len) };

    let packet = packet::TcpPacket::hack_mut(packet);

    packet.set_src_port(src_port);
    packet.set_dst_port(dst.1);

    builder(packet)
  };

  let awkward_checksum_fixer: for<'p> |&'p mut ipv4::packet::V| -> result::Result<(), E> = | packet |
  {
    let packet = packet::TcpPacket::hack_mut(packet);

    match src_addr {
      Some(addr) => if addr != packet.get_src_addr() {
        return Err(upcaster(Error::RouteBrokeConnection))
      },
      _ => ()
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
