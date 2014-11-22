use packet::TcpPacket;
use super::TCP_BUF_SIZE;

const TCP_RECV_WND_INIT : uint = TCP_BUF_SIZE;

pub struct RecvMgr {

  recv_NXT : uint,   // Expected next sequence number received
  recv_WND : uint,   // Recv Window Size
  //recv_UP  : uint, // Recv Urgent Pointer
  recv_ISN : uint,   // Recv Initial Sequence Number
  
  // TODO: make PriorityQueue
  // Queue of received packets whose data has not been consumed yet
  packets: Vec<TcpPacket>,

  // Offset to start reading from within the first packet on the queue
  packet_offset: uint,
}

impl RecvMgr {
  
  //****** Kernel API **********//
  pub fn recv(&self, packet: TcpPacket, notify_read: || -> (), notify_write: || -> ()) {
    //TODO: add packet to packets
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

  //TODO: all the things
  pub fn new() -> RecvMgr {
    RecvMgr {
      packets: vec!(),
      packet_offset: 0,
      recv_NXT : 0u,
      recv_WND : TCP_RECV_WND_INIT,
      recv_ISN : 0u, // TODO: get from handshake
    }
  }

}
