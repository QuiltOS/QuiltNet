use std::fmt;
use std::mem::transmute;
use std::num::Int;
use std::vec::Vec;

use super::Addr;
use super::parse_addr_unsafe;
use super::write_addr;

#[deriving(PartialEq, PartialOrd, Eq, Ord,
           Clone, Show)]
pub struct V { buf: Vec<u8> }

pub struct A { buf:    [u8] }


impl V {
  pub fn new(buf: Vec<u8>) -> V {
    V { buf: buf }
  }

  /// NOT CHECKSUMED!
  fn new_with_header(ip:                 Addr,
                     protocol:           u8,
                     expected_body_size: Option<u16>) -> V
  {
    let mut buf: Vec<u8> = Vec::with_capacity(MIN_HDR_LEN_BYTES as uint
                                              + expected_body_size.unwrap_or(0) as uint);
    unsafe { buf.set_len(MIN_HDR_LEN_BYTES as uint); }
    let mut packet = V::new(buf);
    {
      let s = packet.borrow_mut();
      const SENTINAL8:  u8  = 0b_1000_0001;
      const SENTINAL16: u16 = 0b_1100_1000_0000_0011;
      const SENTINAL32: u32 = 0b_1110_0000_0000_0000_0000_0000_0000_0111;
      *(s.cast_h_mut()) = Header {
        version_ihl:           SENTINAL8,  // SET LATER
        ///////////////////////////////////// Internet header length
        type_of_service:       SENTINAL8,  // SET LATER
        total_length:          SENTINAL16, // DO NOT SET
        identification:        SENTINAL16,
        flags_fragment_offset: SENTINAL16, // SET LATER
        ///////////////////////////////////// Fragment Offset
        time_to_live:          128,
        protocol:              protocol,
        header_checksum:       SENTINAL16, // DO NOT SET
        source_address:        SENTINAL32, // DO NOT SET
        destination_address:   SENTINAL32, // SET LATER
      };
      s.set_version(4);
      s.set_header_length(MIN_HDR_LEN_WORDS);
      s.set_type_of_service(Precedence::Routine, ServiceFlags::empty());
      s.set_flags_fragment_offset(DONT_FRAGMENT, 0);
      s.set_destination(ip);
    }
    packet
  }

  pub fn new_with_builder
    <Err, Accum>
    (ip:                 Addr,
     protocol:           u8,
     expected_body_size: Option<u16>,
     builder:            |&mut V| -> Result<Accum, Err>)
     -> Result<(Accum, V), Err>
  {
    let mut packet = V::new_with_header(ip, protocol, expected_body_size);

    let accum = try!(builder(&mut packet));

    let len = packet.borrow().as_slice().len() as u16;

    // once the new error handling libs land
    // this can be return Err(...) instead
    assert!(len > MIN_HDR_LEN_BYTES);

    // now fix header and checksum
    {
      let s = packet.borrow_mut();
      s.cast_h_mut().total_length = len.to_be();
      s.update_checksum();
    }
    Ok((accum, packet))
  }

  pub fn as_vec(&self) -> &Vec<u8> { &self.buf }

  pub fn as_mut_vec(&mut self) -> &mut Vec<u8> { &mut self.buf }

  pub fn to_vec(self) -> Vec<u8> { self.buf }

  pub fn borrow(&self) -> &A { unsafe { transmute(self.buf.as_slice()) } }

  pub fn borrow_mut(&mut self) -> &mut A { unsafe { transmute(self.buf.as_mut_slice()) } }
}

pub const MIN_HDR_LEN_BITS:  u32 = MIN_HDR_LEN_WORDS as u32 * 32;
pub const MIN_HDR_LEN_BYTES: u16 = MIN_HDR_LEN_WORDS as u16 * 4;
pub const MIN_HDR_LEN_WORDS: u8  = 5;

///   From RFC 791
///
///    0                   1                   2                   3
///    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |Version|  IHL  |Type of Service|          Total Length         |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |         Identification        |Flags|      Fragment Offset    |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |  Time to Live |    Protocol   |         Header Checksum       |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                       Source Address                          |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                    Destination Address                        |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                    Options                    |    Padding    |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

#[repr(packed)]
#[unstable]
pub struct Header {
  pub version_ihl:           u8,   // IP version (= 4)
  /////////////////////////////////// Internet header length
  pub type_of_service:       u8,   // Type of service
  pub total_length:          u16,  // Total length in octets
  pub identification:        u16,  // Identification
  pub flags_fragment_offset: u16,  // 3-bits Flags
  /////////////////////////////////// Fragment Offset
  pub time_to_live:          u8,   // Time To Live
  pub protocol:              u8,   // Protocol
  pub header_checksum:       u16,  // Checksum
  pub source_address:        u32,  // Source Address
  pub destination_address:   u32,  // Destination Address
}

#[repr(u8)]
pub enum Precedence {
  NetworkControl      = 0b_111_00000,
  InternetworkControl = 0b_110_00000,
  CriticEpc           = 0b_101_00000,
  FlashOverride       = 0b_100_00000,
  Flash               = 0b_011_00000,
  Immediate           = 0b_010_00000,
  Priority            = 0b_001_00000,
  Routine             = 0b_000_00000,
}

bitflags! {
  flags ServiceFlags: u8 {
    const LOW_DELAY       = 0b000_100_00,
    const HIGH_THROUGHPUT = 0b000_010_00,
    const HIGH_RELIABILTY = 0b000_001_00,
  }
}

bitflags! {
  flags IpFlags: u16 {
    const DONT_FRAGMENT  = 0b010_00000_00000000,
    const MORE_FRAGMENTS = 0b001_00000_00000000,
  }
}


struct CastHelper {
  header: Header,
  _rest: [u8], // used to make it a DST
}

impl A {

  pub fn as_slice(&self) -> &[u8] {
    unsafe { transmute(self) }
  }

  pub fn as_mut_slice(&mut self) -> &mut [u8] {
    unsafe { transmute(self) }
  }

  pub fn cast_h(&self) -> &Header {
    &    unsafe { transmute::<_,&CastHelper>(self) }.header
  }

  pub fn cast_h_mut(&mut self) -> &mut Header {
    &mut unsafe { transmute::<_,&mut CastHelper>(self) }.header
  }


  pub fn new(buf: &[u8]) -> &A {
    unsafe { transmute(buf) }
  }

  pub fn new_mut(buf: &mut [u8]) -> &mut A {
    unsafe { transmute(buf) }
  }



  pub fn get_version(&self) -> u8 { self.buf[0] >> 4 }
  pub fn set_version(&mut self, v: u8) {
    const MASK: u8 = 0b1111_0000;
    assert!(v & MASK == 0);
    self.buf[0] &= !MASK; // clear lower bits
    self.buf[0] |= v << 4;
  }

  pub fn get_header_length(&self) -> u8 { self.buf[0] & 0b0000_1111 }
  pub fn set_header_length(&mut self, hl: u8) {
    const MASK: u8 = 0b1111_0000;
    assert!(hl & MASK == 0);
    self.buf[0] &= MASK; // clear lower bits
    self.buf[0] |= hl;
  }

  pub fn hdr_bytes(&self) -> uint { self.get_header_length() as uint * 4 }

  pub fn get_total_length(&    self) -> u16 { Int::from_be(self.cast_h()    .total_length) }
  pub fn set_total_length(&mut self, v: u16)             { self.cast_h_mut().total_length = v.to_be(); }


  pub fn get_type_of_service(&self) -> (Precedence, ServiceFlags) {
    const MASK: u8 = 0b111_00000;
    let tos = self.cast_h().type_of_service;
    ( unsafe { ::std::mem::transmute(tos & MASK) },
      ServiceFlags { bits: tos & !MASK } )
  }
  pub fn set_type_of_service(&mut self, prec: Precedence, flags: ServiceFlags) {
    self.cast_h_mut().type_of_service = prec as u8 | flags.bits;
  }


  pub fn get_identification(&    self) -> u16 { Int::from_be(self.cast_h()    .identification) }
  pub fn set_identification(&mut self, v: u16)             { self.cast_h_mut().identification = v.to_be(); }


  pub fn get_flags_fragment_offset(&self) -> (IpFlags, u16) {
    let ffo = Int::from_be(self.cast_h().flags_fragment_offset);
    const MASK: u16 = 0b111_00000_00000000;
    ( unsafe { ::std::mem::transmute(ffo & MASK) },
      ffo & !MASK)
  }
  pub fn set_flags_fragment_offset(&mut self, flags: IpFlags, offset: u16) {
    assert!(0 == (offset & 0b111_00000_00000000));
    self.cast_h_mut().flags_fragment_offset = (flags.bits | offset).to_be();
  }


  pub fn get_time_to_live(&    self) -> u8  { self.cast_h()    .time_to_live }
  pub fn set_time_to_live(&mut self, v: u8) { self.cast_h_mut().time_to_live = v; }

  pub fn get_protocol(&    self) -> u8  { self.cast_h()    .protocol }
  pub fn set_protocol(&mut self, v: u8) { self.cast_h_mut().protocol = v; }

  pub fn get_header_checksum(&    self) -> u16 { Int::from_be(self.cast_h()    .header_checksum) }
  pub fn set_header_checksum(&mut self, v: u16)             { self.cast_h_mut().header_checksum = v.to_be(); }

  pub fn get_source(&self) -> Addr { parse_addr_unsafe(self.buf[12..16]) }
  pub fn set_source(&mut self, a: Addr) {
    // TODO: report assign slices lack of doing anything
    let [a, b, c, d] = write_addr(a);
    self.buf[12] = a;
    self.buf[13] = b;
    self.buf[14] = c;
    self.buf[15] = d;
  }

  pub fn get_destination(&self) -> Addr { parse_addr_unsafe(self.buf[16..20]) }
  pub fn set_destination(&mut self, a: Addr) {
    let [a, b, c, d] = write_addr(a);
    self.buf[16] = a;
    self.buf[17] = b;
    self.buf[18] = c;
    self.buf[19] = d;
  }

  // Eh, todo. Iterator over IpOptions?
  //pub fn options(&self) -> ... {  }

  pub fn get_payload(&self) -> &[u8] {
    if self.get_total_length() as uint > self.buf.len() {
      self.buf[self.hdr_bytes() as uint..]
    } else {
      self.buf[self.hdr_bytes() as uint..self.get_total_length() as uint]
    }
  }

  pub fn get_payload_mut(&mut self) -> &mut [u8] {
    let start = self.hdr_bytes() as uint;
    if self.get_total_length() as uint > self.buf.len() {
      self.buf[mut start..]
    } else {
      let end = self.get_total_length() as uint;
      self.buf[mut start..end]
    }
  }

  /// returns native endian
  pub fn make_header_checksum(&self) -> u16 {
    let u16s: &[u16] = unsafe { transmute(self.as_slice()) };

    // TODO: Factor out singleton iterator
    let temp: [u16, ..1] = [0]; // for checkum field itself

    // [..12] to make sure body is excluded,
    // and also because length might be incorrect from transmute
    let iter = u16s[0..5].iter()
      .chain(temp.iter())
      .chain(u16s[6..10].iter());

    make_checksum(iter.map(|x| Int::from_be(*x)))
  }

  pub fn update_checksum(&mut self) {
    let cs = self.make_header_checksum();
    self.set_header_checksum(cs);
  }
}

impl fmt::Show for A {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f,
           "Ip  | ver {} | {} | Tos {} | Len {}  |\n    | FId {}    |   off {} |\n    | ttl {} | proto {} | sum {} |\n    | Src {}   | Dst {} |",
           self.get_version(),
           self.get_header_length(),
           self.cast_h().type_of_service,
           self.get_total_length(),

           self.get_identification(),
           self.get_flags_fragment_offset().val1(),

           self.get_time_to_live(),
           self.get_protocol(),
           self.get_header_checksum(),

           self.get_source(),
           self.get_destination())
  }
}


#[deriving(PartialEq, PartialOrd, Eq, Ord,
           Clone, Show)]
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

pub fn validate(buf: &[u8]) -> Result<(), BadPacket>
{
  // have to check this first to avoid out-of-bounds panic on version check
  if buf.len() < 1 {
    return Err(BadPacket::TooShort(buf.len()))
  }

  let packet = A::new(buf);

  // try to do this early as possible in case is other type of packet
  if packet.get_version() != 4 {
    return Err(BadPacket::BadVersion(packet.get_version()))
  };

  // then this so other header indexing doesn't packet
  if packet.as_slice().len() != packet.get_total_length() as uint {
    return Err(BadPacket::BadPacketLength(packet.as_slice().len(),
                                          packet.get_total_length()))
  };


  if packet.hdr_bytes() > packet.as_slice().len()
  {
    return Err(BadPacket::HeaderTooLong(packet.hdr_bytes(),
                                        packet.as_slice().len()))
  };
  if packet.hdr_bytes() < MIN_HDR_LEN_BYTES as uint
  {
    return Err(BadPacket::HeaderTooShort(packet.hdr_bytes()))
  };

  {
    let expected = packet.make_header_checksum();
    let got      = packet.get_header_checksum();
    if expected != got
    {
      return Err(BadPacket::BadChecksum(expected, got));
    }
  };

  Ok(())
}

/// assumes and returns native byte order
fn make_checksum<I>(iter: I) -> u16
  where I: Iterator<u16>
{
  let mut sum = iter
    .map(|x| x as u32)
    .fold(0, |old, cur| old + cur);

  debug!("Untruncated checksum is: {:032b}", sum);

  while sum >> 16 != 0 {
    sum = (sum & 0x0000FFFF) + (sum >> 16);
  }

  // align debug messages
  debug!("Truncated checksum is:   {:032b}", sum);

  !(sum as u16)
}
