use std::collections::BinaryHeap;
use std::io::BufWriter;
use std::fmt;
use std::cmp;

use super::super::tcb::{TCP_BUF_SIZE, TcbState, mod_in_interval};

#[deriving(Show, Eq, PartialEq)]
struct BufEntry {
  pub seq: u32,

  // Index within data corresponding to above sequence #
  pub hdr_offset: u32,

  // Index within data at which consumption should start
  pub consume_offset: u32,
  pub data: Vec<u8>
} 

// Holds (Seq#, Header Offset, Data) for pakcets in packet queue
impl BufEntry {

  fn len(&self) -> u32 {
    self.data.len() - hdr_offset
  }

  //TODO: mod??
  pub fn covers(&self, other: &BufEntry) -> bool {
    self.contains(other.seq) && self.contains(other.seq + other.len())
  }

  pub fn contains(&self, seq: u32) -> bool {
    self.seq <= seq && self.seq + self.len() >= seq
  }

  pub fn slice_until_seq(&self, seq: u32) -> &[u8] {
    self.data[self.get_cur_ix()..cmp::min(self.len(), get_ix(seq))]
  }

  fn get_end_seq(&self) -> u32 {
    self.seq + self.len()
  }

  fn get_cur_ix(&self) -> uint {
    self.hdr_offset + self.consume_offset
  }

  fn get_ix(&self, seq: u32) -> uint {
    assert!(self.contains(seq));
    self.hdr_offset + (seq - self.seq)
  }
}

#[deriving(Show)]
pub struct RecvMgr {

  // Queue of received packets whose data has not been consumed yet
  packets: Vec<BufEntry>,

  // Offset to start reading from within the first packet on the queue
  packet_offset: u32,

  // Next sequence number that the user will consume
  pub usr_NXT: u32,

  // Virtual size of buffer
  pub size: u16, 
}


/// The Data Structure that provides functionality to iterate in-order through data
/// sent or received in pieces that may not have been in order
impl RecvMgr {
  
  //****** Methods Kernel Calls *********//
  pub fn new(&self) -> RecvMgr {
    RecvMgr {
      packets: vec!(),
      packet_offset: 0u32,
      usr_XT: 0u32,
      size: TCP_BUF_SIZE,
    }
  }


  // Adds packet to receive queue
  // TODO: return boolean to tell us whether usr_NXT is overlapped, so we can notify canRead?
  #[inline]
  pub fn add_packet(&mut self, seq: u32, offset: u32, data: Vec<u8>) { 
    let entry = BufEntry {
      seq: seq,
      hdr_offset: offset,
      data: data
    };
    match self.find_ix(entry) {
      None => (),
      Some(ix) => self.packets.insert(ix, entry)
    }
  }


  // TODO: Binary search using OrdSlicePrelude
  // Finds index where given entry belongs in sorted sequence if it is not a duplicate
  fn find_ix(&self, entry: &BufEntry) -> Option<uint> {
    for (ix, e) in self.packets.iter().enumerate() {
      if e.seq > entry.seq {
        Some(ix)
      } else if e.seq == entry.seq {
        if !e.covers(entry) { 
          Some(ix)
        }  else {
          None
        }
      }
    }
    //Just push
    Some(self.packets.len())
  }

  pub fn iter_to(&self, seq: u32) -> Iterator<&[u8]> {
    NonConsumingIterator {
      buf: self,
      until: seq,
      cur_ix: 0u,
      next_seq: buf.consume_NXT,
    }
  }

  pub fn consume_to(&mut self, seq: u32) -> Iterator<&[u8]> {
    ConsumingIterator {
      buf: self,
      until: seq,
      next_seq: buf.consume_NXT,
    }
  }

  pub fn iter_all(&self) -> Iterator<Vec<u8>> {
    self.packets.iter().map( |entry| entry.data)
  }

}


impl Iterator<&[u8]> for ConsumingIterator {
  // Call to next:
  // - Checks first packet to see if it is contiguous
  // - if so
  //  - read as much as we need, if whole packet, pop it
  fn next(&mut self) -> Option<&[u8]> {
    
    // Short-circuit if goal already met
    if self.until <= next_seq {
      None
    }

    let mut destroy = false;
    let mut result  = None;
    match self.buf.packets {
      [] => (),
      [first, ..] => {

        // Packet is contiguous w/ previous
        if first.contains(self.next_seq) {

          // Packet contains all the data we need
          if first.contains(self.until) {
            
            result = Some(first.slice_to_seq(self.until));
          } else {
            result = Some(first.slice_to_seq(self.until));
            destroy = true;
          } 
        } else {
          None
        } 
      }
    }

    self.next_seq = self.until;
    if destroy {
      self.buf.packets.remove(0);
    } 
    result
  }
}

impl Iterator<&[u8]> for NonConsumingIterator {
  // A Call to next does:
  // - Looks at the current Vec (by self.cur_ix and 
  fn next(&mut self) -> Option<&[u8]> {
    match self.buf.packets[self] {
      [] => None,
      [first, ..] => {
        if !first.contains(self.until) {
            
        } else {
        }
      }
    }
  }
}


/*
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

  /// Returns sequence number of last byte of contiguous block of data starting at usr_NXT
  /// NOTE: could pop off all intermediate packets and put them in a user-dedicated buffer
  ///   -> This would make additions faster in the case of a lazy user who never reads
  /// TODO: use the iterator abstraction we should use for the above
  pub fn shift_nxt(&mut self) -> u32 {
    0u32  
  }

  /// Returns a new RecvMgr, complete with:
  /// - An empty priority queue, ready to sort TcpPackets based on beginning sequence number
  /// - Pointer to the next sequence number the user will consume, starting at 0
  pub fn new() -> RecvMgr {
    RecvMgr {
      packets: BinaryHeap::new(),
      packet_offset: 0,
      usr_NXT: 0u32,
      size: TCP_BUF_SIZE,
    }
  }

}

/// Pretty Print Impl for BinaryHeap for debug
impl fmt::Show for BinaryHeap<TcpPacket> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "RecvQueue: {}", self.clone().into_sorted_vec())
  }
}
*/
