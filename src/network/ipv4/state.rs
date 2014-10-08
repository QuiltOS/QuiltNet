use std::collections::hashmap::HashMap;
use std::io::net::ip::IpAddr;
use std::mem::size_of;
use std::sync::RWLock;


use packet::parser;

use data_link::{DLInterface, DLHandler};

use super::receive::{IPProtocolHandler, ProtocolTable};

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
pub type InterfaceTable = HashMap<IpAddr, uint>;

pub struct IPState {
    pub routes:            RWLock<RoutingTable>,
    pub interfaces:        InterfaceTable,
    pub interface_vec:     Vec<(IpAddr, IpAddr, Box<DLInterface+'static>)>,
    // JOHN: local_vips is the same as .keys() on interfaces
    // quicker to just index vector
    pub protocol_handlers: ProtocolTable,
    // Identification counter? increased with each packet sent out,
    // used in Identification header for fragmentation purposes
}

impl IPState {
    pub fn new(interface_vec:   Vec<(IpAddr, IpAddr, Box<DLInterface + 'static>)>,
               interface_table: InterfaceTable)
               -> IPState
    {
        IPState {
            routes:            RWLock::new(HashMap::new()),
            interfaces:        interface_table,
            interface_vec:     interface_vec,
            protocol_handlers: Vec::with_capacity(size_of::<u8>()),
        }

    }

    /// Returns DLInterface struct for the requested interface
    pub fn get_interface<'a> (&'a self, interface_ix: uint) -> Option<&'a (IpAddr, IpAddr, Box<DLInterface>)> {
        self.interface_vec.as_slice().get(interface_ix)
    }


// //// Fill interface table
// fn make_interface_table(interface_vec: Vec<(IpAddr, IpAddr, Box<DLInterface>)>,
//                         table: InterfaceTable) {
//     for &(local_vip, nbr_vip, ref interface) in interface_vec.iter() {
//         table.insert(local_vip, (nbr_vip, interface));
//     }
// }
    ///TODO: need index on interfaces by int?
    pub fn up(&self, interface : int){
        
    }

    ///TODO: need index on interfaces by int?
    pub fn down(&self, interface : int){

    }
} 
