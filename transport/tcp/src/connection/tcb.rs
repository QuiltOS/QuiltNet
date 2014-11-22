use ringbuf::RingBuf;
use packet::TcpPacket;
use super::manager::recv::RecvMgr;
use super::manager::send::SendMgr;


/// Encapsulates all connection state and data structures
/// TODO: build this once socket is opened so you can accumulate 
/// synchronization state during handshake!
/// TODO: move all send/recv specific state into send/recvMgr
pub struct TCB {

  // Buffers
  recv_mgr : RecvMgr, 
  send_mgr : SendMgr,

}

impl TCB {
  //TODO: pass in connection state vars
  // recvISN, sendISN
  pub fn new() -> TCB {
    TCB {
      recv_mgr : RecvMgr::new(),
      send_mgr : SendMgr::new(),
    }
  }

  // ******** TCP Module API ***********************************//

  /// Receive logic for TCP packet
  /// TODO: return type? - maybe hint for ACK response
  pub fn recv(&self, packet: TcpPacket, notify_read:  || -> (), notify_write: || -> ()) {
    self.recv_mgr.recv(packet, notify_read, notify_write)
  }

  /// Send logic for TCP Packets 
  fn send_packet(&self, packet: TcpPacket, notify: || -> ()) {
  }

  // ********* Userland API ************************************//

  /// Read as many bytes as we can into buf from received TCP data
  /// until we get to n bytes, starting from the next unconsumed 
  /// sequence number.
  ///
  /// Returns the number of bytes read
  pub fn read(&self, buf: &mut [u8], n: uint) -> uint {
    self.recv_mgr.read(buf, n)
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

}
