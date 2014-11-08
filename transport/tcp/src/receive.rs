use std::io::IoResult;
use std::sync::Arc;

use misc::interface::MyFn;

use network::ipv4::{
  mod,
  control,
  send,
};
use network::ipv4::strategy::RoutingTable;


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
  Ok(())
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
