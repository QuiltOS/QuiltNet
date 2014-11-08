use std::io::{
  BufReader,
  BufWriter,
  MemWriter,
  SeekSet,

  IoError,
  IoResult,
};
use std::mem::{transmute, size_of};

use network::ipv4::Addr;
use network::ipv4::packet;


// Length of TCP header in bytes
pub const TCP_HDR_LEN: uint = 20;

pub struct TcpPacket {
  ip: packet::V
}

#[deriving(PartialEq, PartialOrd, Eq, Ord,
           Clone, Show)]
// TODO: I just copied this from IP, feel free to add or remove error cases Anson.
/// Where there are two fields: expected, then got.
pub enum BadPacket {
  TooShort(uint),              // header cannot fit

  BadVersion(u8),              // isn't 4
  BadPacketLength(uint, u16),  // not what it really is

  HeaderTooLong(uint, uint),   // header declared shorter than min or longer than body
  HeaderTooShort(uint),        // header declared shorter than min or longer than body

  BadChecksum(u16, u16),
  BadOptions,
}

impl TcpPacket {

  pub fn new(ip_packet: packet::V) -> TcpPacket {
    TcpPacket {
      ip: ip_packet
    }
  }

  pub fn validate(ip_packet: &packet::A) -> Result<(), BadPacket> {
    Ok(())
  }

  /// Returns slice containing TCP packet
  fn get_tcp(&self) -> &[u8] {
    self.ip.borrow().get_payload()
  }

  /// Returns mutable slice containing TCP packet body
  fn get_tcp_mut(&mut self) -> &mut [u8] {
    self.ip.borrow_mut().get_payload_mut()
  }

  /// Returns immutable slice containing TCP packet header
  /// NOTE: assumes no TCP options
  fn tcp_hdr(&self) -> &[u8] {
    self.get_tcp()[..TCP_HDR_LEN]
  }

  /// Returns mutable slice containing TCP packet header
  /// NOTE: assumes no TCP options
  fn tcp_hdr_mut(&mut self) -> &mut [u8] {
    self.get_tcp_mut()[mut ..TCP_HDR_LEN]
  }


  // 4-tuple info
  pub fn get_src_addr(&self) -> Addr {
    self.ip.borrow().get_source()
  }
  pub fn get_src_port(&self) -> u16 {
    //TODO:
    0
  }
  pub fn set_src_port(&mut self, port: u16) {
    //TODO:
  }
  pub fn get_dst_addr(&self) -> Addr {
    self.ip.borrow().get_destination()
  }
  pub fn get_dst_port(&self) -> u16 {
    //TODO:
    0
  }
  pub fn set_dst_port(&mut self, port: u16) {
    //TODO
  }

  // Control Flags
  pub fn is_ack(&self) -> bool {
    //TODO:
    false
  }
  pub fn set_ack(&mut self) {
    //TODO
  }
  pub fn is_syn(&self) -> bool {
    //TODO:
    false
  }
  pub fn set_syn(&mut self) {
    //TODO
  }
  pub fn is_fin(&self) -> bool {
    //TODO:
    false
  }
  pub fn set_fin(&mut self) {
    //TODO
  }

  // Not sure if this is used
  pub fn is_rst(&self) -> bool {
    //TODO:
    false
  }
  pub fn set_rst(&mut self) {
    //TODO
  }

  // Other TCP data
  pub fn get_hdr_size(&self) -> u8 { // really u8
    //TODO:
    0
  }

  // Sequence Number Ops
  pub fn get_seq_num(&self) -> u32 {
    //TODO:
    0
  }
  pub fn set_seq_num(&mut self, seq_num: u16) {
    //TODO:
  }

  // Acknowledgement Number Ops
  pub fn get_ack_num(&self) -> u32 {
    //TODO:
    assert!(self.is_ack());
    0
  }
  pub fn set_ack_num(&mut self, ack_num: u16) {
    //TODO:
  }

  // Checksum Ops
  pub fn get_checksum(&self) -> u16 {
    //TODO:
    0
  }
  pub fn compute_checksum(&self) -> u16 {
    //TODO:
    0
  }
  pub fn set_checksum(&mut self, checksum: u16) {
    //TODO:
  }
  pub fn update_checksum(&mut self) {
    let cs = self.compute_checksum();
    self.set_checksum(cs);
  }

  pub fn get_payload(&self) -> &[u8] {
    self.get_tcp()[TCP_HDR_LEN..]
  }

  pub fn get_mut_payload(&mut self) -> &mut[u8] {
    self.get_tcp_mut()[mut TCP_HDR_LEN..]
  }

}
