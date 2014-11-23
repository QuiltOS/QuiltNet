use std::collections::BinaryHeap;
use std::io::BufWriter;

use packet::{TcpPacket, ACK, SYN};
use super::super::tcb::{TCP_BUF_SIZE, TcbState, mod_in_interval};

//TODO: move all this into TCB, too much shared state and dependencies b/w Recv/Send logic
pub struct RecvMgr {

  // Queue of received packets whose data has not been consumed yet
  packets: BinaryHeap<TcpPacket>,

  // Offset to start reading from within the first packet on the queue
  packet_offset: u32,

  // Next sequence number that the user will consume
  usr_NXT: u32,
}

impl RecvMgr {
  

  // Adds packet to receive queue
  pub fn add_packet(&mut self, packet: TcpPacket) { 
    self.packets.push(packet);
  }

  //****** Userland API *******//
  /// Blocking read 
  /// TODO: mutate TCB state?
  pub fn read(&mut self, buf: &mut [u8], n: uint) -> uint {

    let mut bufw = BufWriter::new(buf);
    let mut bytes_read = 0u;
    let mut more_contiguous = true;

    // Read through packets, copying into user's buf, until we have read n bytes
    // or we run out of contiguous data
    while bytes_read < n && more_contiguous {
      let (packet_read, more_contiguous, to_pop) = self.read_top(&mut bufw, n - bytes_read);

      // Pop off this packet if it was consumed
      if to_pop {
        self.packets.pop();
      }

      bytes_read += packet_read as uint;
      self.usr_NXT += packet_read;
    }
    bytes_read
  }

  // Tries to copy n bytes from next packet in receive queue
  // Pops packet off queue if fully consumed
  // Resets self.packet_offset appropriately
  fn read_top(&mut self, bufw: &mut BufWriter, n: uint) -> (u32, bool, bool) {
      match self.packets.top() {
        None => (0, false, false),
        Some(next_packet) => {

          let seg_SEQ  = next_packet.get_seq_num();
          let seg_LEN = next_packet.get_body_len();
          
          // Packet is next in order
          if mod_in_interval(seg_SEQ, seg_LEN, self.usr_NXT) {

            if seg_LEN as u32 - self.packet_offset > n as u32 {

              bufw.write(next_packet.get_payload()[self.packet_offset as uint..self.packet_offset as uint + n]);
              self.packet_offset += n as u32;

              // Read n bytes, may have additional data, didn't consume this packet entirely
              return (n as u32, true, false)
            } else {
              
              let rest : u32 = seg_LEN as u32 - self.packet_offset;

              // Read until end of packet
              bufw.write(next_packet.get_payload()[self.packet_offset as uint..(self.packet_offset + rest) as uint]);

              // Reset packet offset
              self.packet_offset = 0u32;

              // We may have more contiguous data, read again to find out
              return (rest, true, true)
            }

            // This packet arrived out of order
          } else {
            return (0, false, false)
          }
        }
      }
  }
  
  /// Returns a new RecvMgr, complete with:
  /// - An empty priority queue, ready to sort TcpPackets based on beginning sequence number
  /// - Pointer to the next sequence number the user will consume, starting at 0
  pub fn new() -> RecvMgr {
    RecvMgr {
      packets: BinaryHeap::new(),
      packet_offset: 0,
      usr_NXT: 0u32,
    }
  }

}
