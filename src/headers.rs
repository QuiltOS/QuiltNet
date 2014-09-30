
struct IPHeader {
    
    ihl : uint,         // Internet Header Length 
    version : uint,     // Header Version = 4
    tos : u8,           // Type of Service
    tot_len : u16,      // Total Length
    id : u16,           // Identification
    frag_offset : u16,  // Fragment Offset (leading 3 bits are flags)
    ttl : u8,           // Time To Live
    protocol : u8,      // Protocol
    checksum : u16,     // Checksum
    src_addr : u32,     // Source Address
    dst_addr : u32,     // Destination Address

    //TODO: make IPOptions struct?
    // options : IPOptions // Options - variable length, padded
};

// IP Parameters (from /usr/include/netinet/ip.h)
static MAX_TTL : u8 = 255;          // maximum time to live 
static DEFAULT_TTL : u8 = 64;       // default ttl, from RFC 1340
static TTL_DEC : u8 = 1;            // ttl decrement when forwarding

//TODO: set MTU for real
static MTU : int = 1024;            // Maximum Transfer Unit 
static MAX_PKT_SIZE : int = 65_535  // Maximum Packet Size
static MSS : int = 576;             // default maximum segment size

// IP Fragmenting Parameters (from /usr/include/netinet/ip.h)
// for def of slowhz, see 
// lists.freebsd.org/pipermail/freebsd-net/2011-July/029440.html
static FRAG_TTL : u8 = 60;          // default ttl for frags, slowhz
