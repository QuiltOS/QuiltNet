use ringbuf::RingBuf;
use packet::{TcpPacket, ACK};
use super::manager::recv::RecvMgr;
use super::manager::send::SendMgr;

pub const TCP_BUF_SIZE : u32 = 1u32 << 16u;
pub const TCP_RECV_WND_INIT : u32 = TCP_BUF_SIZE;

pub struct TcbState {
  pub recv_NXT : u32,   // Expected next sequence number received
  pub recv_WND : u32,   // Recv Window Size
  //recv_UP  : u32, // Recv Urgent Pointer
  pub recv_ISN : u32,   // Recv Initial Sequence Number
  
  // State Variables
  // TODO: sizes of all these
  pub send_UNA : u32,   // Oldest unacknowledged sequence number
  pub send_NXT : u32,   // Next Sequence Number to be Sent
  pub send_WND : u32,   // Send Window Size
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
        send_WND : 0u32,
        send_WL1 : 0u32,
        send_WL2 : 0u32,
        send_ISN : our_isn 
    }
  }
}


/// Encapsulates all connection state and data structures
/// TODO: build this once socket is opened so you can accumulate 
/// synchronization state during handshake!
/// TODO: move all send/recv specific state into send/recvMgr
pub struct TCB {

  // Buffers
  recv_mgr : RecvMgr, 
  send_mgr : SendMgr,
  state:     TcbState,
}

impl TCB {
  //TODO: pass in connection state vars
  // recvISN, sendISN
  pub fn new(our_isn: u32, their_isn: u32) -> TCB {
    TCB {
      recv_mgr : RecvMgr::new(),
      send_mgr : SendMgr::new(),

      // TODO: make sure init is consistent with how handshake inits state 
      state    : TcbState::new(our_isn, their_isn),
    }
  }

  // ******** TCP Module API ***********************************//

  /// Receive logic for TCP packet
  /// TODO: return type? - maybe hint for ACK response
  pub fn recv(&mut self, packet: &TcpPacket, notify_read:  || -> (), notify_write: || -> ()) {
    // If ACK
    //  - Is acceptable?
    //    -> if (send_WL1 < SEG.SEQ) or (send_WL1 == SEG.SEQ && 
    //    ->  
    //  - Else trash
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

  fn recv_acceptable(&self, packet: &TcpPacket) -> bool{
    //TODO: checks for acceptability
    // ACK num in SND window
    // SEQ num in RCV window
    // ???

    // Check if 'acceptable ack'
    // NOTE Should we trash packet if not? Still might be valid data
    if packet.flags().contains(ACK){

      // Check SND.UNA < SEG.ACK =< SND.NXT
      return mod_in_interval(self.state.send_UNA, packet.get_ack_num(), self.state.send_NXT)
    } else {
      //TODO
      return true
    }

    //TODO: check if packet overlaps acceptable receive window of sequenc numbers 

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

