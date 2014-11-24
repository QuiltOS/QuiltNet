use ringbuf::RingBuf;
use packet::{mod, TcpPacket};
use super::manager::recv::RecvMgr;
use super::manager::send::SendMgr;

pub const TCP_BUF_SIZE : u16 = ((1u32 << 16u) - 1u32) as u16;
pub const TCP_RECV_WND_INIT : u16 = TCP_BUF_SIZE;

#[deriving(Show)]
pub struct TcbState {
  pub recv_NXT : u32,   // Expected next sequence number received
  pub recv_WND : u16,   // Recv Window Size
  //recv_UP  : u32, // Recv Urgent Pointer
  pub recv_ISN : u32,   // Recv Initial Sequence Number

  // State Variables
  // TODO: sizes of all these
  pub send_UNA : u32,   // Oldest unacknowledged sequence number
  pub send_NXT : u32,   // Next Sequence Number to be Sent
  pub send_WND : u16,   // Send Window Size
  //send_UP  : u32, // send urgent pointer
  pub send_WL1 : u32,   // Seq number for last window update
  pub send_WL2 : u32,   // Ack number for last window update
  pub send_ISN : u32,   // Send Initial Sequence Number


}

impl TcbState {

  pub fn new(our_isn: u32, their_isn: u32) -> TcbState {
    TcbState {
        recv_NXT : 0u32,
        recv_WND : TCP_RECV_WND_INIT,
        recv_ISN : their_isn,
        send_UNA : 0u32,
        send_NXT : 0u32,
        send_WND : 0u16,
        send_WL1 : 0u32,
        send_WL2 : 0u32,
        send_ISN : our_isn
    }
  }
}


/// Encapsulates all connection state and data structures
pub struct TCB {

  // Buffers
  recv_mgr : RecvMgr,
  send_mgr : SendMgr,
  state:     TcbState,
}

impl TCB {
  pub fn new(our_isn: u32, their_isn: u32) -> TCB {
    TCB {
      recv_mgr : RecvMgr::new(),
      send_mgr : SendMgr::new(),
      state    : TcbState::new(our_isn, their_isn),
    }
  }

  // ******** TCP Module API ***********************************//

  /// Receive logic for TCP packet
  /// TODO: return type? - maybe hint for ACK response
  pub fn recv(&mut self, packet: TcpPacket) -> (bool, bool)
  {
    debug!("In the TCB!");

    let mut can_read  = false;
    let mut can_write = false;
    // Validate (ACK, SEQ in appropriate intervals)
    // Is duplicate? -> trash TODO: quick duplicate detection
    if self.validate_packet_state(&packet) && !self.is_duplicate(&packet){

      let seg_SEQ = packet.get_seq_num();
      let seg_WND = packet.get_window_size();

      // If ACK
      match packet.get_ack_num() {
        None => (),
        Some(seg_ACK) => {
          // If ACKing new data
          // FIXME: is this covered in validate_packet_state()?
          if seg_ACK > self.state.send_UNA {
            //    -> SND.UNA = SEG.ACK
            self.state.send_UNA = seg_ACK;
            //    -> if (send_WL1 < SEG.SEQ) or (send_WL1 == SEG.SEQ && send_WL2 <= SEG.ACK)
            if (self.state.send_WL1 < seg_SEQ) ||
              (self.state.send_WL1 == seg_SEQ && self.mod_leq(self.state.send_WL2, seg_ACK))
            {
              self.state.send_WND = seg_WND;
              self.state.send_WL1 = seg_SEQ;
              self.state.send_WL2 = seg_ACK;
            } else {
              debug!("valid packet with shrinking ACK?");
              //    -> else
              //      -> don't care
            }
            // We have more space to write into now
            can_write = true;
              //    | OR start timer to buffer ACKS to only annouce the highest ACK instead of notifying X
              //    times
          }
        }
      }

      let contains_nxt = self.contains_recv_nxt(&packet);
      // Handle data now
      // TODO: gonna need to own that packet
      // we know this isn't a dup from above
      self.recv_mgr.add_packet(packet);

      if contains_nxt {
        //  -> update RCV.WND = (2^16-1) - (recv_NXT - usr_NXT) [unconsumed space in buf]
        self.state.recv_NXT = self.recv_mgr.shift_nxt();

        // Got more data, try to read again
        can_read = true;
      }
      //  ->TODO Will require method in RecvMgr that can iterate over first contiguous block from RCV.NXT
      //  -  and do things at each packet (copy to buf), optionally deleting from queue once consumed

      // -> Send ACK TODO: efficient way of ACKing - single ACK value at any time, should just update
      // ACK number in timer
    } else {
      debug!("Invalid Packet State: TCB: {}, SEG:<ACK:{}, SEQ:{}>", self.state,
             packet.get_ack_num(),
             packet.get_seq_num());
    }
    (can_read, can_write)
  }

  //TODO
  fn validate_packet_state(&self, packet: &TcpPacket) -> bool {
    true
  }

  //TODO
  fn is_duplicate(&self, packet: &TcpPacket) -> bool {
    false
  }

  #[inline]
  /// Returns the receive window: the number of bytes in the receive buffer that are not reserved
  /// by unconsumed, sequentially ordered data
  fn get_rcv_window(&self) -> u16 {
    self.recv_mgr.size - (self.state.recv_NXT - self.recv_mgr.usr_NXT) as u16
  }

  #[inline]
  fn contains_recv_nxt(&self, packet: &TcpPacket) -> bool {
    mod_in_interval(packet.get_seq_num(), packet.get_seq_num() + packet.get_body_len(), self.state.recv_NXT)
  }

  /// Send logic for TCP Packets
  fn send_packet(&self, packet: TcpPacket) {
  }

  // ********* Userland API ************************************//

  /// Read as many bytes as we can into buf from received TCP data
  /// until we get to n bytes, starting from the next unconsumed
  /// sequence number.
  ///
  /// Returns the number of bytes read
  pub fn read(&mut self, buf: &mut [u8], n: uint) -> uint {
    self.recv_mgr.read(buf, n)
    //TODO: update recv_WND += n
  }


  /// Write as many bytes as we can into our TCP send buffer
  /// from buf, starting with `start`
  ///
  /// Returning the number of bytes we were able to successfully write
  /// NOTE: this is less than n when
  ///               n > (SND.UNA + SND.WND ) - SND.NXT
  /// TODO: all the things
  pub fn send(&self, buf: &[u8], start: u32, n: uint) -> uint {
    self.send_mgr.send(buf, start, n)
  }

  //TODO: how the fuck are we supposed to figure out n <= m (mod x)...
  fn mod_leq(&self, n: u32, m: u32) -> bool {
    n <= m || (n - m > (1 << 16))
  }
}

// Checks if n in (s, e] (mod 2^32)
// TODO: should be in utils or something?
#[inline]
pub fn mod_in_interval(s: u32, e: u32, n: u32) -> bool {
  if e < s {

    // interval is wrapped around
    s < n || n <= e
  } else {

    // Plain old interval
    s < n && n <= e
  }
}
