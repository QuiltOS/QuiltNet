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

  pub fn hack(ip_packet: &packet::V) -> &TcpPacket {
    unsafe {transmute(ip_packet) }
  }

  pub fn hack_mut(ip_packet: &mut packet::V) -> &mut TcpPacket {
    unsafe {transmute(ip_packet) }
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
    Int::from_be(get_multibyte(self.tcp_hdr(), 0, 2) as u16)
  }
  pub fn set_src_port(&mut self, port: u16) {
    //TODO:
  }
  pub fn get_dst_addr(&self) -> Addr {
    self.ip.borrow().get_destination()
  }
  pub fn get_dst_port(&self) -> u16 {
    Int::from_be(get_multibyte(self.tcp_hdr(), 2, 2) as u16)
  }
  pub fn set_dst_port(&mut self, port: u16) {
    //TODO
  }

  // Control Flags
  pub fn is_ack(&self) -> bool {
    self.tcp_hdr()[13] & 16 != 0
  }
  pub fn set_ack(&mut self) {
    //TODO
  }
  // Not sure if this is used
  pub fn is_rst(&self) -> bool {
    self.tcp_hdr()[13] & 4 != 0
  }
  pub fn set_rst(&mut self) {
    //TODO
  }
  pub fn is_syn(&self) -> bool {
    self.tcp_hdr()[13] & 2 != 0
  }
  pub fn set_syn(&mut self) {
    //TODO
  }
  pub fn is_fin(&self) -> bool {
    self.tcp_hdr()[13] & 1 != 0
  }
  pub fn set_fin(&mut self) {
    //TODO
  }


  // Recv Window size
  pub fn get_window_size(&self) -> u16 {
    Int::from_be(get_multibyte(self.tcp_hdr(), 14, 2)) as u16
  }
  pub fn set_window_size(&mut self) {
    //TODO
  }

  // AKA data offset: 4 bytes
  pub fn get_hdr_size(&self) -> u8 {
    (self.tcp_hdr()[12] >> 4) as u8
  }

  // Sequence Number Ops
  pub fn get_seq_num(&self) -> u32 {
    // assert!(self.is_seq())
    Int::from_be(get_multibyte(self.tcp_hdr(), 4, 4)) as u32
  }
  pub fn set_seq_num(&mut self, seq_num: u16) {
    //TODO:
  }

  // Acknowledgement Number Ops
  pub fn get_ack_num(&self) -> u32 {
    assert!(self.is_ack());
    Int::from_be(get_multibyte(self.tcp_hdr(), 8, 4)) as u32
  }
  pub fn set_ack_num(&mut self, ack_num: u16) {
    //TODO:
  }

  // Checksum Ops
  pub fn get_checksum(&self) -> u16 {
    Int::from_be(get_multibyte(self.tcp_hdr(), 16, 2)) as u16
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

  /// Returns TCP payload as slice
  pub fn get_payload(&self) -> &[u8] {
    self.get_tcp()[TCP_HDR_LEN..]
  }

  /// Returns TCP payload as mut slice
  pub fn get_mut_payload(&mut self) -> &mut[u8] {
    self.get_tcp_mut()[mut TCP_HDR_LEN..]
  }

}

// Reads int in BE order
// TODO: make sure endianness is alright: I'm calling Int::from_be on result
// to read multibyte fields from packet headers
fn get_multibyte(buf: &[u8], start_ix: uint, len: uint) -> int {
  let mut res = 0i;

  // Iterate over bytes in reverse order, increasing significance
  for (ix, v) in  buf[start_ix..start_ix + len].iter().rev().enumerate() {
     res |= ((*v) as int << (8 * ix));
  }
  res
}

#[cfg(test)]
mod test {
  use super::get_multibyte;

  #[test]
  fn byte(){
    let b = [1u8]; // 0x1 == 1
    assert!(get_multibyte(b, 0, 1) == 1i);
  }
  
  #[test]
  fn short(){
    let b = [1u8, 255u8]; // 0x1_11111111 == 511
    assert!(get_multibyte(b, 0, 1) == 1i);
    assert!(get_multibyte(b, 0, 2) == 511i);
  }

  #[test]
  fn u32(){
    let b = [10u8, 112u8, 1u8, 123u8]; // 0x00001010_01110000_00000001_01111011 == 511
    assert!(get_multibyte(b, 0, 1) == 10i);
    assert!(get_multibyte(b, 0, 2) == 2672i);
    assert!(get_multibyte(b, 0, 3) == 684033i);
    assert!(get_multibyte(b, 0, 4) == 175112571);
    assert!(false);
  }

}
