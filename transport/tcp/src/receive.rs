use std::io::{
  IoError,
  IoResult,
};
use std::sync::Arc;

use misc::interface::MyFn;

use network::ipv4::{
  mod,
  control,
  send,
};
use network::ipv4::strategy::RoutingTable;

use super::packet::TcpPacket;

struct Handler { state: Arc<super::Table> }

impl MyFn<(ipv4::packet::V,), ()> for Handler
{
  fn call(&self, (packet,):(ipv4::packet::V,)) {
    handle(&*self.state, packet).unwrap(/* "Failure handling incomming IP Packet" */);
  }
}

fn handle(state:  &super::Table,
          packet: ipv4::packet::V)
          -> IoResult<()>
{
  try!(TcpPacket::validate(packet.borrow())
       .map_err(|_| IoError::last_error())); // some random error

  let packet = TcpPacket::new(packet);
  let dst_port = packet.get_dst_port();

  let lock = state.0.read();

  let sub_table = match lock.get(&dst_port) {
    Some(p) => p,
    None    => return Err(IoError::last_error()),
  };

  let src_info = (packet.get_src_addr(),
                  packet.get_src_port());

  match sub_table.connections.read().get(&src_info) {
    Some(connection) => {
      // TODO: handle with connection
      Ok(())
    },
    None => { // no existing connection, let's see if we have a listener
      let listener = match sub_table.listener {
        Some(ref l) => l,
        None    => return Err(IoError::last_error()),
      };
      // TODO: handle with listener
      Ok(())
    },
  }
}

/// Registers protocol handler for incomming RIP packets.
pub fn register<A>(ip_state:  &ipv4::State<A>,
                   tcp_state: Arc<super::Table>)
  where A: RoutingTable
{
  control::register_protocol_handler(
    ip_state,
    super::PROTOCOL,
    box Handler { state: tcp_state })
}
