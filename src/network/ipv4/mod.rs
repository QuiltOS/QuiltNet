use std::collections::hashmap::{HashMap, HashSet};
use std::io::net::ip::IpAddr;
use std::mem::size_of;
use std::sync::RWLock;

use data_link::DLInterface;


//use data_link::{DLInterface, DLHandler};

use self::packet::{IPHeader, IPPacket};
use self::receive::{IPProtocolHandler, ProtocolTable};


pub mod packet;
pub mod send;
pub mod receive;

pub struct RoutingRow {
    pub cost:      u8,     // How many hops
    pub next_hop:  IpAddr, // which link-layer interface to use
}

// key: IP we want to reach
pub type RoutingTable = HashMap<IpAddr, RoutingRow>;

// key:   adjacent ip (next hop)
// value:  which one of our IPs we put as the src address
//         which interface we send the packet with
//pub type InterfaceTable = HashMap<IpAddr, (IpAddr, Box<DLInterface+'static>)>;
pub type InterfaceTable<'a> = HashMap<IpAddr, (IpAddr, &'a Box<DLInterface+'static>)>;

pub struct IPState<'a> {
    routes:     RWLock<RoutingTable>,
    interfaces: &'a InterfaceTable<'a>,
    interface_vec: &'a Vec<(IpAddr, IpAddr, Box<DLInterface+'static>)>,
    // JOHN: local_vips is the same as .keys() on interfaces
    // quicker to just index vector
    protocol_handlers: ProtocolTable,
    // Identification counter? increased with each packet sent out, 
    // used in Identification header for fragmentation purposes
}

impl<'a> IPState<'a> {
    pub fn new(interface_vec: &'a Vec<(IpAddr, IpAddr, Box<DLInterface>)>, interface_table: &'a InterfaceTable<'a>) -> IPState<'a> {
        
        // Add interface entries to table
        make_interface_table(interface_vec, interface_table);
        IPState {
            routes:     RWLock::new(HashMap::new()),
            interface_vec: interface_vec,
            interfaces: interface_table, 
            protocol_handlers: Vec::with_capacity(size_of::<u8>()),
        }

    }

    /// Returns DLInterface struct for the requested interface
    pub fn get_interface<'a> (&'a self, interface_ix: uint) -> Option<&'a (IpAddr, IpAddr, Box<DLInterface>)> {
        if interface_ix < self.interface_vec.len() {
            Some(self.interface_vec.get(interface_ix))
        } else {
            None
        }
    }
}

/// Fill interface table 
fn make_interface_table<'a>(interface_vec: &'a Vec<(IpAddr, IpAddr, Box<DLInterface>)>, table: &'a mut InterfaceTable<'a>){
    for &(local_vip, nbr_vip, ref interface) in interface_vec.iter() {
        table.insert(local_vip, (nbr_vip, interface));
    }
}

