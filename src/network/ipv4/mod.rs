use std::collections::hashmap::{HashMap, HashSet};
use std::sync::RWLock;

use self::packet::{IPAddr, IPHeader, IPPacket};
use self::protocol::IPProtocolHandler;
use interface::{Interface, Handler};


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
pub type InterfaceTable = HashMap<IPAddr, (IPAddr, Box<Interface+'static>)>;

pub struct IPState {
    routes:     RWLock<RoutingTable>,
    interfaces: InterfaceTable,
    local_vips: HashSet<IPAddr>,
    protocol_handlers: HashMap<u8, Vec<IPProtocolHandler>>,
}

impl IPState {
    pub fn new(interfaces: InterfaceTable) -> IPState {
        IPState {
            routes:     RWLock::new(HashMap::new()),
            interfaces: interfaces,
        }
    }
}

/*
pub trait IpModule {
    pub fn send(&self, vip : IPAddr, proto : u8, data : [u8]);
    pub fn with_interfaces(&self, fun : |&InterfaceTable|);
    pub fn with_routes(&self, fun : |&RoutingTable|);
    pub fn recv(&self, data : [u8]);
    pub fn up(&self, interface : int);
    pub fn down(&self, interface : int);
}

pub struct IpModuleReal {
    routes : RoutingTable,
    interfaces : InterfaceTable,
    protocol_handlers : HashMap<u8, Vec<Sender<IpPacket>>>
}


pub trait IpModule {
    
    /// Registers the handler for the given protocol
    /// This handler will be sent every packet that arrives with the given protocol
    pub fn register_handler(&self, protocol : u8, handler : IIpHandler){
        self.protocol_handlers.insert_or_update_with(protocol, vec!(handler.tx), 
                                                |k, v| v.push(handler.tx));
    }

    /// Forwards this packet to all registered handlers
    pub fn recv(&self, data : [u8]) -> (){

        // Parse packet from bytes
        match self.parse_packet(data) {
            Some(packet) => {

                // Lookup handlers for this protocol
                match self.protocol_handlers.find(packet.header.protocol) {
                    Some(handlers) => {

                        // Send packet to each of our handler tasks
                        for handler in handlers.iter() {
                            handler.send(packet);
                        }
                    }
                    None => () // no handlers for this packet's protocol
                }
            },
            None => () // drop malformed packet
        }
    }

    /// Sends data to the given VIP on the given protocol
    pub fn send(&self, vip : IPAddr, proto : u8, data : [u8]) -> (){
        match self.routes.find(vip) {
            Some(next_hop) => {
                match self.interfaces.find(next_hop) {
                    Some(interface) => {
                        let packet = self.make_packet(vip, proto, data);
                        interface.send(packet.to_bytes());
                    },
                    None => () // no interface for this hop TODO: check if possible
                }
            },
            None => () // drop, since requested VIP is unreachable
        }
    }

    ///TODO: need index on interfaces by int?
    pub fn up(&self, interface : int){
        
    }

    ///TODO: need index on interfaces by int?
    pub fn down(&self, interface : int){

    }
} 
*/
