extern crate packet;

use std::collections::hashmap::HashMap;
use std::io::net::ip::IpAddr;
use std::iter::FromIterator;
use std::mem::size_of;
use std::sync::{Arc, RWLock};

use interface::{MyFn, Handler};

use packet::ipv4::V as Ip;

use data_link::{DLInterface, DLHandler};

use self::strategy::RoutingTable;

pub mod control;
pub mod send;
pub mod receive;

pub mod strategy;

// key:    adjacent ip (next hop)
// value:  which one of our Ips we put as the src address
//         which interface we send the packet with
//pub type InterfaceTable = HashMap<IpAddr, (IpAddr, Box<DLInterface+'static>)>;
pub type InterfaceTable = HashMap<IpAddr, uint>;

pub type InterfaceRow = (IpAddr, IpAddr, RWLock<Box<DLInterface + Send + Sync + 'static>>);

// TODO: use Box<[u8]> instead of Vec<u8>
// TODO: real network card may consolidate multiple packets per interrupt
pub type IpHandler = //Handler<Ip>;
    Box<MyFn<(Ip,), ()> + Send + Sync + 'static>;

pub type ProtocolTable = Vec<Vec<IpHandler>>;

pub struct IpState<A> where A: RoutingTable {
    pub routes:            A,
    pub ip_to_interface:   InterfaceTable,
    pub interfaces:        Vec<InterfaceRow>,
    pub protocol_handlers: RWLock<ProtocolTable>,
    // Identification counter? increased with each packet sent out,
    // used in Identification header for fragmentation purposes
}

impl<A> IpState<A> where A: RoutingTable
{
    pub fn new(ip_to_interface_vec: Vec<InterfaceRow>) -> Arc<IpState<A>>
    {
        use std::iter::count;
        use std::iter::Repeat;

        let ip_to_interface = {
            let ip_to_interface_iter = ip_to_interface_vec.iter()
                .zip(count(0, 1))
                .map(|(&(_, ref dst, _), ix)| (dst.clone(), ix));
            FromIterator::from_iter(ip_to_interface_iter)
        };

        let routes = strategy::RoutingTable::init(ip_to_interface_vec.as_slice());

        let state = Arc::new(IpState {
            routes:            routes,
            ip_to_interface:   ip_to_interface,
            interfaces:        ip_to_interface_vec,
            // handlers are not clonable, so the nice ways of doing this do not work
            protocol_handlers: RWLock::new(vec!(
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),

                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),

                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),

                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),


                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),

                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),

                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),

                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!(),
                vec!(), vec!(), vec!(), vec!(),   vec!(), vec!(), vec!(), vec!())),
        });

        for &(_, _, ref interface) in state.interfaces.iter() {
            use self::receive::make_receive_callback;
            (*interface.write())
                .update_recv_handler(make_receive_callback(state.clone()));
        }

        state
    }

    /// Returns DLInterface struct for the requested interface
    pub fn get_interface<'a> (&'a self, interface_ix: uint) -> Option<&'a InterfaceRow> {
        self.interfaces.as_slice().get(interface_ix)
    }
}
