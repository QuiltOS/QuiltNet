use std::collections::hashmap::HashMap;
use std::io::net::ip::IpAddr;
use std::iter::FromIterator;
use std::mem::size_of;
use std::sync::{Arc, RWLock};

use interface::Handler;

use packet::ipv4::V as Ip;

use data_link::{DLInterface, DLHandler};

pub struct RoutingRow {
    pub cost:      u8,          // How many hops
    pub next_hop:  IpAddr,      // Which link-layer interface to use
    pub learned_from: IpAddr,   // Who we learned this route from (used in split-horizon)
}

// key: IP we want to reach
pub type RoutingTable = HashMap<IpAddr, RoutingRow>;

// key:   adjacent ip (next hop)
// value:  which one of our IPs we put as the src address
//         which interface we send the packet with
//pub type InterfaceTable = HashMap<IpAddr, (IpAddr, Box<DLInterface+'static>)>;
pub type InterfaceTable = HashMap<IpAddr, uint>;

pub type InterfaceRow = (IpAddr, IpAddr, RWLock<Box<DLInterface + Send + Sync + 'static>>);

// TODO: use Box<[u8]> instead of Vec<u8>
// TODO: real network card may consolidate multiple packets per interrupt
// TODO: lifetime for IPState probably needs fixing
// TODO: Make some Sender type
pub type IPProtocolHandler = //Handler<Ip>;
    Box<Fn<(Ip,), ()> + Send + Sync + 'static>;

pub type ProtocolTable = Vec<Vec<IPProtocolHandler>>;

pub struct IPState {
    pub routes:            RWLock<RoutingTable>,
    pub interfaces:        InterfaceTable,
    pub interface_vec:     Vec<InterfaceRow>,
    // JOHN: local_vips is the same as .keys() on interfaces
    // quicker to just index vector
    pub protocol_handlers: RWLock<ProtocolTable>,
    // Identification counter? increased with each packet sent out,
    // used in Identification header for fragmentation purposes
}

impl IPState {
    pub fn new(interfaces_vec: Vec<InterfaceRow>) -> Arc<IPState>
    {
        use std::iter::count;
        let interfaces = {
            let interfaces_iter = interfaces_vec.iter()
                .zip(count(0, 1))
                .map(|(&(ref src, _, _), ix)| (src.clone(), ix));
            FromIterator::from_iter(interfaces_iter)
        };

        let state = Arc::new(IPState {
            routes:            RWLock::new(HashMap::new()),
            interfaces:        interfaces,
            interface_vec:     interfaces_vec,
            protocol_handlers: RWLock::new(Vec::with_capacity(size_of::<u8>())),
        });

        for &(_, _, ref interface) in state.interface_vec.iter() {
            use super::receive::make_receive_callback;
            (*interface.write())
                .update_recv_handler(make_receive_callback(state.clone()));
        }

        state
    }

    /// Returns DLInterface struct for the requested interface
    pub fn get_interface<'a> (&'a self, interface_ix: uint) -> Option<&'a InterfaceRow> {
        self.interface_vec.as_slice().get(interface_ix)
    }
}
