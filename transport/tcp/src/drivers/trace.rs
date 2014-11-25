use time;
use std::sync::Arc;

use network::ipv4::{mod, control, send};
use packet::TcpPacket;

const recv_str : &'static str = "RECV";
const send_str : &'static str = "SEND";

// Register RECV callback
pub fn register<A>(state: &Arc<::State<A>>)
  where A: ipv4::strategy::RoutingTable
{
  control::register_protocol_handler(
    &*state.ip,
    ::PROTOCOL,
    {
      let state = state.clone();
      box move | packet: ipv4::packet::V | {
        log_trace(&TcpPacket::new(packet), true);
      }
    })
}

pub fn log_trace(tcp_packet: &TcpPacket, is_recv: bool) {
  let action = if is_recv { recv_str } else { send_str };
  println!("[TRACE][{}][ns:{}][seq:{}][ack:{}][len:{}]", action, 
                             (time::now_utc().tm_sec as u64) * 100_000_000u64 + time::now_utc().tm_nsec as u64,
                             tcp_packet.get_seq_num(),
                             tcp_packet.get_ack_num(),
                             tcp_packet.get_body_len(),
                             );
}
