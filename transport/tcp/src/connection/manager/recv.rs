use std::collections::BinaryHeap;

use packet::{TcpPacket, ACK, SYN};
use super::super::tcb::{TCP_BUF_SIZE, TcbState};


// Checks if n in (s, e] (mod 2^32)
#[inline]
fn mod_in_interval(s: u32, e: u32, n: u32) -> bool {
  if e < s {

    // interval is wrapped around
    s < n || n <= e
  } else {

    // Plain old interval
    s < n && n <= e
  }
}

//TODO: move all this into TCB, too much shared state and dependencies b/w Recv/Send logic
pub struct RecvMgr {

  // Queue of received packets whose data has not been consumed yet
  packets: BinaryHeap<TcpPacket>,

  // Offset to start reading from within the first packet on the queue
  packet_offset: uint,
}

impl RecvMgr {
  
  //****** Kernel API **********//
  pub fn recv(&self, tcb_state: &mut TcbState, packet: &TcpPacket, notify_read: || -> (), notify_write: || -> ()) {
    if self.recv_acceptable(tcb_state, packet) {

      //TODO: add packet to packets
      //TODO: if packet contains NXT, call notify_read
      //TODO: if ACK > SND.UNA, update UNA = ACK, call notify_write

    } else {

    }

    //notify_read called if: we receive new packet overlapping RCV.NXT, moving our RCV.WND
    //notify_write called if: we receive an ACK that moves our SND.WND
  }


  //****** Userland API *******//
  /// Blocking read 
  /// TODO: pass in TCB as mut
  pub fn read(&self, buf: &mut [u8], n: uint) -> uint {
    //TODO: read available consecutive data until we read n bytes
    //Move RCV.WND as we read
    0u
  }

  fn recv_acceptable(&self, tcb_state: &TcbState, packet: &TcpPacket) -> bool {
    //TODO: checks for acceptability
    // ACK num in SND window
    // SEQ num in RCV window
    // ???

  // Check if 'acceptable ack'
  // NOTE Should we trash packet if not? Still might be valid data
  if packet.flags().contains(ACK){

    // Check SND.UNA < SEG.ACK =< SND.NXT
    return mod_in_interval(tcb_state.send_UNA, packet.get_ack_num(), tcb_state.send_NXT)
  } else {
    //TODO
    return true
  }

  //TODO: check if packet overlaps acceptable receive window of sequenc numbers 


  }

  //TODO: all the things
  pub fn new() -> RecvMgr {
    RecvMgr {
      packets: BinaryHeap::new(),
      packet_offset: 0,
    }
  }

}

