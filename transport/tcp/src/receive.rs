use std::io::{
  IoError,
  IoResult,
};
use std::sync::Arc;

use network::ipv4::{
  mod,
  control,
  send,
};
use network::ipv4::strategy::RoutingTable;

use super::packet::TcpPacket;



fn handle<A>(state:  &Arc<::State<A>>,
             packet: ipv4::packet::V)
  where A: RoutingTable
{
  let packet = match TcpPacket::validate(packet) {
    Ok(p)  => p,
    Err(e) => {
      debug!("TCP packet invalid because {}", e);
      return;
    },
  };

  debug!("Got TCP Packet: {}", &packet);

  let dst_port = packet.get_dst_port();

  let sub_table = match state.tcp.get(&dst_port) {
    Some(p) => p,
    None    => {
      debug!("no sub-table--definitely no listener or connection to handle this")
      return;
    },
  };

  let src_info = (packet.get_src_addr(),
                  packet.get_src_port());

  match sub_table.connections.get(&src_info) {
    Some(connection) => {
      debug!("existing connection found to handle this! (might be closed)");
      super::connection::trans(
        &mut *connection.write(),
        &**state,
        packet)
    },
    None => {
      debug!("no existing connection, let's see if we have a listener");
      super::listener::trans(
        &mut *sub_table.listener.write(),
        state,
        packet)
    },
  }
}

/// Registers protocol handler for incomming RIP packets.
pub fn register<A>(state: &Arc<super::State<A>>)
  where A: RoutingTable
{
  control::register_protocol_handler(
    &*state.ip,
    super::PROTOCOL,
    {
      let state = state.clone();
      box move | packet: ipv4::packet::V | {
        handle(&state, packet);
      }
    })
}
