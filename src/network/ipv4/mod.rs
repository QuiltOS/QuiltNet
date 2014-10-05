use std::collections::hashmap::{HashMap, HashSet};
use std::mem::size_of;
use std::sync::RWLock;

use data_link::{DLInterface, DLHandler};

use self::packet::{IPAddr, IPHeader, IPPacket};
use self::receive::{IPProtocolHandler, ProtocolTable};


pub mod packet;
pub mod send;
pub mod receive;

pub struct RoutingRow {
    pub cost:      u8,     // How many hops
    pub next_hop:  IPAddr, // which link-layer interface to use
}

// key: IP we want to reach
pub type RoutingTable = HashMap<IPAddr, RoutingRow>;

// key:   adjacent ip (next hop)
// value:  which one of our IPs we put as the src address
//         which interface we send the packet with
pub type InterfaceTable = HashMap<IPAddr, (IPAddr, Box<DLInterface+'static>)>;

pub struct IPState {
    routes:     RWLock<RoutingTable>,
    interfaces: InterfaceTable,
    // JOHN: local_vips is the same as .keys() on interfaces
    // quicker to just index vector
    protocol_handlers: ProtocolTable,
}

impl IPState {
    pub fn new(interfaces: InterfaceTable) -> IPState {
        IPState {
            routes:     RWLock::new(HashMap::new()),
            interfaces: interfaces,
            protocol_handlers: Vec::with_capacity(size_of::<u8>()),
        }
    }
}
