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

struct Handler<A>  where A: RoutingTable {
  state: Arc<super::State<A>>,
}

impl<A> MyFn<(ipv4::packet::V,), ()> for Handler<A>
  where A: RoutingTable
{
  fn call(&self, (packet,):(ipv4::packet::V,)) {
    handle(&*self.state, packet);
  }
}

fn handle<A>(state:  &::State<A>,
             packet: ipv4::packet::V)
  where A: RoutingTable
{
  match TcpPacket::validate(packet.borrow()) {
    Ok(_)  => (),
    Err(e) => debug!("TCP packet invalid because {}", e),
  };

  let packet = TcpPacket::new(packet);
  let dst_port = packet.get_dst_port();

  let lock = state.tcp.read();

  let sub_table = match lock.get(&dst_port) {
    Some(p) => p,
    None    => return,
  };

  let src_info = (packet.get_src_addr(),
                  packet.get_src_port());

  match sub_table.connections.read().get(&src_info) {
    // existing connetion found!
    Some(connection) => super::connection::state::trans(
      &mut *connection.write(),
      state,
      packet),
    // no existing connection, let's see if we have a listener
    None => match sub_table.listener {
      None               => return,
      Some(ref listener) => super::listener::state::trans(
        &mut *listener.write(),
        state,
        packet),
    },
  }
}

/// Registers protocol handler for incomming RIP packets.
pub fn register<A>(state: Arc<super::State<A>>)
  where A: RoutingTable
{
  control::register_protocol_handler(
    &*state.ip,
    super::PROTOCOL,
    box Handler { state: state.clone() })
}
