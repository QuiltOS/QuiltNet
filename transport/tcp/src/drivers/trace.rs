use network::ipv4::{strategy, control, packet, State};
use packet::TcpPacket;

/// Runs simple debug handler, printing out all packets received for the given protocols
pub fn register<A>(state: &State<A>,
                   protocols: Vec<u8>)
  where A: strategy::RoutingTable
{
  // For each protocol we want to listen on, register handler
  for protocol in protocols.iter() {
    control::register_protocol_handler(
      state,
      *protocol,
      box |&: packet: packet::V| {
        log_trace(packet);
      })
  }
}

fn log_trace(ip: packet::V) {
  let tcp_packet = TcpPacket::new(ip);
  println!("Packet {}", tcp_packet.get_seq_num());
}
