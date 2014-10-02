pub struct IPAddr(u8, u8, u8, u8);

pub struct IPHeader {

    pub ihl:          u32,    // Internet Header Length
    pub version:      u32,    // Header Version = 4
    pub tos:          u8,     // Type of Service
    pub tot_len:      u16,    // Total Length
    pub id:           u16,    // Identification
    pub frag_offset:  u16,    // Fragment Offset (leading 3 bits are flags)
    pub ttl:          u8,     // Time To Live
    pub protocol:     u8,     // Protocol
    pub checksum:     u16,    // Checksum
    pub src_addr:     IPAddr, // Source Address
    pub dst_addr:     IPAddr, // Destination Address

    //TODO: make IPOptions struct?
    // options: IPOptions // Options - variable length, padded
}

pub static WILDCARD_ADDR: IPAddr = IPAddr(0, 0, 0, 0);

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
