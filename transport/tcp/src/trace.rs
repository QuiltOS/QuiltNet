use time;
use std::sync::Arc;

use network::ipv4::{mod, control, send};
use packet::TcpPacket;

static RECV_STR : &'static str = "RECV";
static SEND_STR : &'static str = "SEND";

pub fn log_trace(tcp_packet: &TcpPacket, is_recv: bool) {
  let action = if is_recv { RECV_STR } else { SEND_STR };
  info!("[TRACE][{}][ns:{}][seq:{}][ack:{}][len:{}][flags:{}][data:{}]",
        action,
        (time::now_utc().tm_sec as u64) * 100_000_000u64 + time::now_utc().tm_nsec as u64,
        tcp_packet.get_seq_num(),
        tcp_packet.get_ack_num(),
        tcp_packet.get_body_len(),
        tcp_packet.flags(),
        match  ::std::str::from_utf8(tcp_packet.get_payload()) {
          None => "binary data, probz too big",
          Some(s) => s
        });
}
