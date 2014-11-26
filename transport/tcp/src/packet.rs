use std::io::{
  BufReader,
  BufWriter,
  MemWriter,
  SeekSet,

  IoError,
  IoResult,
};
use std::mem::{transmute, size_of};
use std::num::Int;
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

  HeaderTooLong(uint, uint),   // header declared shorter than min or longer than body
  HeaderTooShort(uint),        // header declared shorter than min or longer than body

  BadChecksum(u16, u16),
  BadOptions,
}

bitflags! {
#[deriving(Show)]
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

  pub fn validate(ip: packet::V) -> Result<TcpPacket, BadPacket>
  {
    // have to check this first to avoid out-of-bounds panic on version check
    if ip.borrow().get_total_length() < TCP_HDR_LEN as u16 + packet::MIN_HDR_LEN_8S {
      return Err(BadPacket::TooShort(ip.borrow().get_total_length() as uint))
    }

    let packet = TcpPacket::new(ip);

    // this should be true as long as IP does it's job and our CODE is correct
    // therefore is assert, not check
    assert_eq!(packet.ip.borrow().get_total_length() as uint - packet.ip.borrow().hdr_bytes() as uint,
               packet.get_tcp().len());

    let hdr_len = packet.get_hdr_size() as uint * 4;

    if hdr_len > packet.get_tcp().len()
    {
      return Err(BadPacket::HeaderTooLong(hdr_len,
                                          packet.get_tcp().len()))
    };
    if hdr_len < TCP_HDR_LEN as uint
    {
      return Err(BadPacket::HeaderTooShort(hdr_len))
    };

    {
      let expected = packet.make_header_checksum();
      let got      = packet.get_checksum();
      if expected != got
      {
        return Err(BadPacket::BadChecksum(expected, got));
      }
    };

    Ok(packet)
  }

  pub fn as_vec(&self) -> &Vec<u8> {
    self.ip.as_vec()
  }

  pub fn as_mut_vec(&mut self) -> &mut Vec<u8> {
    self.ip.as_mut_vec()
  }

  pub fn to_vec(self) -> Vec<u8> {
    self.ip.to_vec()
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
  }
  pub fn set_src_port(&mut self, port: u16) {
    BufWriter::new(self.tcp_hdr_mut()[mut 0..2]).write_be_u16(port).unwrap();
  }
  pub fn get_dst_addr(&self) -> Addr {
    self.ip.borrow().get_destination()
  }
  pub fn get_dst_port(&self) -> u16 {
    BufReader::new(self.tcp_hdr()[2..4]).read_be_u16().unwrap()
  }
  pub fn set_dst_port(&mut self, port: u16) {
    BufWriter::new(self.tcp_hdr_mut()[mut 2..4]).write_be_u16(port).unwrap();
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
  }
  pub fn set_window_size(&mut self, window_size: u16) {
    BufWriter::new(self.tcp_hdr_mut()[mut 14..16]).write_be_u16(window_size)
      .unwrap();
  }

  // AKA data offset: 4 bytes
  pub fn get_hdr_size(&self) -> u8 {
    (self.tcp_hdr()[12] >> 4) as u8
  }
  // FIXME: way to ensure this always gets called
  pub fn set_hdr_size(&mut self, size: u8) {
    self.tcp_hdr_mut()[12] = size << 4;
  }

  // Sequence Number Ops
  pub fn get_seq_num(&self) -> u32 {
    // assert!(self.is_seq())
    BufReader::new(self.tcp_hdr()[4..8]).read_be_u32().unwrap()
  }
  pub fn set_seq_num(&mut self, seq_num: u32) {
    BufWriter::new(self.tcp_hdr_mut()[mut 4..8]).write_be_u32(seq_num)
      .unwrap();
  }

  // Acknowledgement Number Ops
  pub fn get_ack_num(&self) -> Option<u32> {
    if self.flags().contains(ACK) {
      Some(BufReader::new(self.tcp_hdr()[8..12]).read_be_u32().unwrap())
    } else {
      None
    }
  }
  pub fn set_ack_num(&mut self, ack_num: u32) {
    self.flags_mut().insert(ACK);
    BufWriter::new(self.tcp_hdr_mut()[mut 8..12]).write_be_u32(ack_num)
      .unwrap();
  }

  // Checksum Ops
  pub fn get_checksum(&self) -> u16 {
    BufReader::new(self.tcp_hdr()[16..18]).read_be_u16().unwrap()
  }
  pub fn set_checksum(&mut self, checksum: u16) {
    BufWriter::new(self.tcp_hdr_mut()[mut 16..18]).write_be_u16(checksum)
      .unwrap();
  }

  /// Returns TCP payload as slice
  pub fn get_payload(&self) -> &[u8] {
    self.get_tcp()[TCP_HDR_LEN..]
  }

  pub fn get_payload_offset(&self) -> uint {
    self.ip.borrow().hdr_bytes() as uint + TCP_HDR_LEN
  }

  /// Returns TCP payload as mut slice
  pub fn get_mut_payload(&mut self) -> &mut[u8] {
    self.get_tcp_mut()[mut TCP_HDR_LEN..]
  }

  /// returns native endian
  pub fn make_header_checksum(&self) -> u16
  {
    // +--------+--------+--------+--------+
    // |           Source Address          |
    // +--------+--------+--------+--------+
    // |         Destination Address       |
    // +--------+--------+--------+--------+
    // |  zero  |  PTCL  |    TCP Length   |
    // +--------+--------+--------+--------+

    let len_tcp_bytes = self.get_tcp().len();

    // the WHOLE IP PACKET
    let bytes: &[u8]  = self.ip.borrow().as_slice();
    let words: &[u16] = unsafe { packet::cast_slice(bytes[..bytes.len() & !1]) };

    // src and dest
    let pseudo1: &[u16] = words[6..10];
    debug!("done magic number index 1");

    let pseudo2: [u16, ..2] = [
      (self.ip.borrow().get_protocol() as u16).to_be(),
      (self.get_tcp().len() as u16).to_be(), // tcp_len
    ];

    // TODO: Factor out singleton iterator
    let zero_checksum_or_last_byte: [u16, ..1] = [
      if len_tcp_bytes % 2 == 0 {
        debug!("no magic number index 2 needed");
        0
      } else { // compensate for last byte
        let n = bytes[bytes.len() - 1] as u16;
        debug!("done magic number index 2");
        n
      }.to_be(),
    ];

    let tcp_words = words[packet::MIN_HDR_LEN_16S as uint..];

    let real1 = tcp_words[..8];
    debug!("done magic number index 3");
    let real2 = tcp_words[9..];
    debug!("done magic number index 4");

    let iter = real1.iter()
      .chain(zero_checksum_or_last_byte.iter())
      .chain(real2.iter())
      .chain(pseudo1.iter())
      .chain(pseudo2.iter());

    packet::make_checksum(iter.map(|x| Int::from_be(*x)))
  }

  pub fn update_checksum(&mut self) {
    let cs = self.make_header_checksum();
    self.set_checksum(cs);
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
    write!(f, "TCP: [Flags:{}] <srcAddr: {}, dstAddr: {}>, |srcPort {}|dstPort {}|\n|Seq# {}|\n|Ack# {}|\n|offset {}|ACK {}|SYN {}|FIN {}|window {}|\n|checksum {}|\n{}", self.tcp_hdr()[13],
           self.get_src_addr(), self.get_dst_addr(),
           self.get_src_port(), self.get_dst_port(),
           self.get_seq_num(),
           self.get_ack_num(),
           self.get_hdr_size(),
           self.flags().contains(ACK), self.flags().contains(SYN), self.flags().contains(FIN),
           self.get_window_size(),
           self.get_checksum(),
           self.get_payload())
  }
}


#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn byte(){
    let b = [1u8]; // 0x1 == 1
    //assert_eq!(get_multibyte(&b, 0, 1),  1i);
  }

  #[test]
  fn short(){
    let b = [1u8, 255u8]; // 0x1_11111111 == 511
    //assert_eq!(get_multibyte(&b, 0, 1), 1i);
    //assert_eq!(get_multibyte(&b, 0, 2), 511i);
  }

  #[test]
  fn u32(){
    let b = [10u8, 112u8, 1u8, 123u8]; // 0x00001010_01110000_00000001_01111011 == 511
    //assert_eq!(get_multibyte(&b, 0, 1), 10i);
    //assert_eq!(get_multibyte(&b, 0, 2), 2672i);
    //assert_eq!(get_multibyte(&b, 0, 3), 684033i);
    //assert_eq!(get_multibyte(&b, 0, 4), 175112571);
  }
}
