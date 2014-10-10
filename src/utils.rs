use std::io::net::ip::{SocketAddr, IpAddr, Ipv4Addr};
use std::io::net::addrinfo;
use std::io::IoResult;

/// Tries to parse Socket Address from <host>:<port> string
pub fn parse_socketaddr(addr_s: &str) -> Option<SocketAddr> {

    // Separate <host> and <port>
    let split_s: Vec<&str> = addr_s.as_slice().trim().split(':').collect();
    match split_s.as_slice() {

        // If two parts, 
        [host, port] => {
            match ::std::u16::parse_bytes(port.as_bytes(), 10) {
            
                // If port is a valid port
                Some(port_num) => 
                    match addrinfo::get_host_addresses(host) {
                        
                        // If we can resolve the hostname to an Ip, return socket address
                        Ok(addrs) => Some(SocketAddr { ip : addrs[0], port: port_num }),
                        Err(_) => None 
                    },
                None => None,
            }
        },
        _ => None
    }
}

/// Tries to parse dotted-quad Ipv4 address from string
pub fn parse_ipv4(vip_s: &str) -> Option<IpAddr> {

    // Separate components of address and try to parse all of them as 8-bit uints
    let quads: Vec<Option<u8>> = vip_s.as_slice().split('.').map(|s| ::std::u8::parse_bytes(s.as_bytes(), 10)).collect();
    match quads.as_slice() {

        // If success, return Ip address
        [Some(q1), Some(q2), Some(q3), Some(q4)] => Some(Ipv4Addr(q1, q2, q3, q4)),
        _ => None
    }
}
