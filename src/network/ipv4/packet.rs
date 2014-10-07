use std::mem::transmute;
use std::io::net::ip::{IpAddr,Ipv4Addr};
/*
#[deriving(Eq)]
#[deriving(PartialEq)]
#[deriving(Hash)]
pub struct IpAddr(u8, u8, u8, u8);
*/

#[repr(packed)]
pub struct IPPacket<'a> {
//    pub header: IPHeader,
    pub data: Box<&'a[u8]>
}

impl<'a> IPPacket<'a> {

    /// Constructs a new packet by building new packet
    /// The packet is safe to cast to a `Box<[u8]>`
    pub fn new<'a>(dest: IpAddr, protocol: u8, data: &'a [u8]) -> Box<IPPacket<'a>> {
        //TODO: implement packing
        // allocate buf of header + len(data) bytes
        // write header, write data into buf
        let buf : Box<Vec<u8>> = make_header(dest, protocol, data);
        buf.append(data); // 5 32-bit words in header = 40 bytes
        box IPPacket {
            data: box buf.as_slice()
        }
    }

    // TODO: implement Clone trait instead
    pub fn clone_box(&self) -> Box<IPPacket> {
       box *self.clone() 
    }
    
    /// Useful to assert that packet is correct
    /// e.g. `try!(verify_packet(&packet));`
    pub fn verify_packet(&self) -> Result<(), ()> {
        Ok(())
    }
    
    
    /// Constructs a new IPPacket from a byte slice
    ///  -> Some(IPPacket) if data is a valid packet
    ///  -> None if data forms invalid packet
    ///
    /// TODO: figure out lifetime stuff of bytes
    pub fn from_bytes<'a> (buf : &'a [u8]) -> Option<Box<IPPacket>> {
        //TODO: actually implement
        None    
    }

    /// Cast the (owned) packet to a byte array
    /// TODO: figure out lifetime stuff of bytes
    pub fn to_bytes<'a>(&'a self) -> Box<&'a [u8]> {
        self.data
        //unsafe { transmute(self) }
        /*
         * let buf: Box<[u8]> = unsafe { transmute(self) };
        buf.to_vec() //TODO(jcericso) Make sure that doesn't copy
        */
    }

    /// TODO:
    pub fn get_header_checksum(&self) -> uint {
        0
    }

    /// TODO:
    pub fn set_header_checksum(&self, checksum: u8) {
        println!("Setting header checksum {}", checksum);
    }

    /// TODO:
    pub fn get_time_to_live(&self) -> u8 {
        0
    }

    /// TODO:
    pub fn set_time_to_live(&self, ttl: u8){
        println!("Setting TTL {}", ttl);
    }

    /// TODO:
    pub fn get_protocol(&self) -> u8 {
        0
    }

    /// TODO:
    pub fn get_destination_address(&self) -> IpAddr {
        Ipv4Addr(127, 0, 0, 0)
    }

}

//TODO: actually write bytes into header
fn make_header(dst: IpAddr, protocol: u8, data: &[u8]) -> Box<Vec<u8>> {
    return box vec!()
}


#[repr(packed)]
pub struct IPHeader {
    pub version_ihl:           u8,                  // IP version (= 4)
    ////////////////////////////////////////////////// Internet header length
    pub type_of_service:       TypeOfService,       // Type of service
    pub total_length:          u16,                 // Total length in octets
    pub identification:        u16,                 // Identification
    pub flags_fragment_offset: FlagsFragmentOffset, // 3-bits Flags
    ////////////////////////////////////////////////// Fragment Offset
    pub time_to_live:          u8,                  // Time To Live
    pub protocol:              u8,                  // Protocol
    pub header_checksum:       u16,                 // Checksum
    pub source_address:        IpAddr,              // Source Address
    pub destination_address:   IpAddr,              // Destination Address

    //TODO: make IPOptions struct?
    // options: IPOptions // Options - variable length, padded
}

impl IPHeader {
    pub fn new(destination: IpAddr, protocol: u8, data: Vec<u8>) -> IPHeader {
        let header = IPHeader {
            version_ihl: 0b0100_0101,                                              // Version = 4, IHL = 5 (since we omit the Options+Padding word)
            type_of_service: make_type_of_service(Routine, LowDelay),              // NOTE: Choose TOS somewhat arbitrarily
            total_length: 0,                                                       // TODO: calculate as sizeof(IPHeader) + sizeof(data)
            identification: 0,                                                     // TODO: incremented counter in IPState
            flags_fragment_offset: make_flags_fragment_offset(DontFragment, 0u16), // NOTE: default to DontFragment, 0 fragment offset
            time_to_live: DEFAULT_TTL,
            protocol: protocol,
            header_checksum: 0,                                                    // placeholder for checksum
            source_address: Ipv4Addr(0,0,0,0),                                       // TODO: state.get_src(destination)
            destination_address: destination, 
        };
        header.add_checksum();
        header
    }

    pub fn add_checksum(&self) {
        //TODO: compute checksum
    }
}

#[repr(u8)]
pub enum Precedence {
    NetworkControl      = 0b111_00000,
    InternetworkControl = 0b110_00000,
    CriticEpc           = 0b101_00000,
    FlashOverride       = 0b100_00000,
    Flash               = 0b011_00000,
    Immediate           = 0b010_00000,
    Priority            = 0b001_00000,
    Routine             = 0b000_00000,
}

bitflags! {
    flags ServiceFlags: u8 {

        static LowDelay          = 0b000_100_00,
        static HighThroughput    = 0b000_010_00,
        static HighReliability   = 0b000_001_00,

        //static NormalDelay       = !LowDelay       .bits,
        //static NormalThroughput  = !HighThroughput .bits,
        //static NormalReliability = !HighReliability.bits,
    }
}

pub struct TypeOfService {
    bits: u8
}

#[inline]
pub fn make_type_of_service(prec: Precedence, flags: ServiceFlags) -> TypeOfService {
    TypeOfService { bits: prec as u8 | flags.bits }
}

#[inline]
pub fn unmake_type_of_service(tos: TypeOfService) -> (Precedence, ServiceFlags) {
    static MASK: u8 = 0b111_00000;
    ( unsafe { ::std::mem::transmute(tos.bits & MASK) },
      ServiceFlags { bits: tos.bits & !MASK })
}


bitflags! {
    flags IPFlags: u16 {

        static DontFragment  = 0b010_00000_00000000,
        static MoreFragments = 0b001_00000_00000000,

    }
}

pub struct FlagsFragmentOffset {
    bits: u16
}

#[inline]
pub fn make_flags_fragment_offset(flags: IPFlags, offset: u16) -> FlagsFragmentOffset {
    assert!(0 == (offset & 0b111_00000_00000000));
    FlagsFragmentOffset { bits: flags.bits | offset }
}

#[inline]
pub fn unmake_flags_fragment_offset(ffo: FlagsFragmentOffset) -> (IPFlags, u16) {
    static MASK: u16 = 0b111_00000_00000000;
    ( unsafe { ::std::mem::transmute(ffo.bits & MASK) },
      ffo.bits & !MASK)
}


pub static WILDCARD_ADDR: IpAddr = Ipv4Addr(0, 0, 0, 0);

// IP Parameters (from /usr/include/netinet/ip.h)
pub static MAX_TTL:       u8     = 255;    // maximum time to live
pub static DEFAULT_TTL:   u8     = 64;     // default ttl, from RFC 1340
pub static TTL_DEC:       u8     = 1;      // ttl decrement when forwarding

                                           //TODO: set MTU for real
pub static MTU:           int    = 1024;   // Maximum Transfer Unit
pub static MAX_PKT_SIZE:  int    = 65_535; // Maximum Packet Size
pub static MSS:           int    = 576;    // default maximum segment size

// IP Fragmenting Parameters (from /usr/include/netinet/ip.h)
// for def of slowhz, see
// lists.freebsd.org/pipermail/freebsd-net/2011-July/029440.html
pub static FRAG_TTL:      u8     = 60;     // default ttl for frags, slowhz
