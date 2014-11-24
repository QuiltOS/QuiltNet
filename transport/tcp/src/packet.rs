use std::io::{
  BufReader,
  BufWriter,
  MemWriter,
  SeekSet,

  IoError,
  IoResult,
};
use std::mem::{transmute, size_of};
use std::fmt;

use network::ipv4::Addr;
use network::ipv4::packet;

// Length of TCP header in bytes
pub const TCP_HDR_LEN: uint = 20;

#[deriving(PartialEq, Eq, Clone)]
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

bitflags! {
  flags Flags: u8 {
    const URG = 0b_00_100000,
    const ACK = 0b_00_010000,
    const PSH = 0b_00_001000,
    const RST = 0b_00_000100,
    const SYN = 0b_00_000010,
    const FIN = 0b_00_000001,
  }
}

impl TcpPacket {

  pub fn new(ip_packet: packet::V) -> TcpPacket {
    TcpPacket { ip: ip_packet }
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

  /// Returns length of the TCP body
  pub fn get_body_len(&self) -> u32 {
    (self.get_tcp().len() - TCP_HDR_LEN) as u32
  }

  // 4-tuple info
  pub fn get_src_addr(&self) -> Addr {
    self.ip.borrow().get_source()
  }
  pub fn get_src_port(&self) -> u16 {
    BufReader::new(self.tcp_hdr()[0..2]).read_be_u16().unwrap()
    //Int::from_be(get_multibyte(self.tcp_hdr(), 0, 2) as u16)
  }
  pub fn set_src_port(&mut self, port: u16) {
    BufWriter::new(self.tcp_hdr_mut()[mut 0..2]).write_be_u16(port);
  }
  pub fn get_dst_addr(&self) -> Addr {
    self.ip.borrow().get_destination()
  }
  pub fn get_dst_port(&self) -> u16 {
    BufReader::new(self.tcp_hdr()[2..4]).read_be_u16().unwrap()
    //Int::from_be(get_multibyte(self.tcp_hdr(), 2, 2) as u16)
  }
  pub fn set_dst_port(&mut self, port: u16) {
    BufWriter::new(self.tcp_hdr_mut()[mut 2..4]).write_be_u16(port);
  }

  // Control Flags
  pub fn flags(&self) -> Flags {
    const MASK: u8 = 0b_00_111111;
    Flags { bits: self.tcp_hdr()[13] & MASK }
  }
  pub fn flags_mut(&mut self) -> &mut Flags{
    unsafe { transmute(&mut self.tcp_hdr_mut()[13]) }
  }


  // Recv Window size
  pub fn get_window_size(&self) -> u16 {
    BufReader::new(self.tcp_hdr()[14..16]).read_be_u16().unwrap()
    //Int::from_be(get_multibyte(self.tcp_hdr(), 14, 2)) as u16
  }
  pub fn set_window_size(&mut self, window_size: u16) {
    BufWriter::new(self.tcp_hdr_mut()[mut 14..16]).write_be_u16(window_size);
  }

  // AKA data offset: 4 bytes
  pub fn get_hdr_size(&self) -> u8 {
    (self.tcp_hdr()[12] >> 4) as u8
  }

  // Sequence Number Ops
  pub fn get_seq_num(&self) -> u32 {
    // assert!(self.is_seq())
    BufReader::new(self.tcp_hdr()[4..9]).read_be_u32().unwrap()
    //Int::from_be(get_multibyte(self.tcp_hdr(), 4, 4)) as u32
  }
  pub fn set_seq_num(&mut self, seq_num: u32) {
    BufWriter::new(self.tcp_hdr_mut()[mut 4..9]).write_be_u32(seq_num);
  }

  // Acknowledgement Number Ops
  pub fn get_ack_num(&self) -> Option<u32> {
    if self.flags().contains(ACK) {
      None
    } else {
      Some(BufReader::new(self.tcp_hdr()[8..13]).read_be_u32().unwrap())
    }
  }
  pub fn set_ack_num(&mut self, ack_num: u32) {
    self.flags_mut().insert(ACK);
    BufWriter::new(self.tcp_hdr_mut()[mut 8..13]).write_be_u32(ack_num);
  }

  // Checksum Ops
  pub fn get_checksum(&self) -> u16 {
    BufReader::new(self.tcp_hdr()[16..18]).read_be_u16().unwrap()
    //Int::from_be(get_multibyte(self.tcp_hdr(), 16, 2)) as u16
  }
  pub fn compute_checksum(&self) -> u16 {
    //TODO:
    0
  }
  pub fn set_checksum(&mut self, checksum: u16) {
    BufWriter::new(self.tcp_hdr_mut()[mut 16..18]).write_be_u16(checksum);
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

// For purposes of sorting by sequence number
impl Ord for TcpPacket {
  fn cmp(&self, other: &TcpPacket) -> Ordering {
    self.get_seq_num().cmp(&other.get_seq_num())
  }
}

impl PartialOrd for TcpPacket {
  fn partial_cmp(&self, other: &TcpPacket) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl fmt::Show for TcpPacket {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "TCP: <srcAddr: {}, dstAddr: {}>, |srcPort {}|dstPort {}|\n|Seq# {}|\n|Ack# {}|\n|offset {}|ACK {}|SYN {}|FIN {}|window {}|\n|checksum {}|",
           self.get_src_addr(), self.get_dst_addr(),
           self.get_src_port(), self.get_dst_port(),
           self.get_seq_num(),
           self.get_ack_num(),
           self.get_hdr_size(),
           self.flags().contains(ACK), self.flags().contains(SYN), self.flags().contains(FIN),
           self.get_window_size(),
           self.get_checksum())
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

/*
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
  }
}
*/
