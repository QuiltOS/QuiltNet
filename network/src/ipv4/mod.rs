use std::collections::hash_map::HashMap;
use std::sync::{Arc, RWLock};

use misc::interface::{Fn, /* Handler */};

use data_link::interface as dl;

use self::strategy::RoutingTable;

pub mod control;
pub mod packet;
pub mod send;
pub mod receive;
pub mod strategy;


#[deriving(PartialEq, PartialOrd, Eq, Ord,
           Clone, Show, Hash)]
pub struct Addr(pub u8, pub u8, pub u8, pub u8);

#[inline]
pub fn parse_addr(&[a, b, c, d]: &[u8, ..4]) -> Addr {
  Addr(a, b, c, d)
}

#[inline]
pub fn parse_addr_unsafe(b: &[u8]) -> Addr {
  Addr(b[0], b[1], b[2], b[3])
}

#[inline]
pub fn write_addr(Addr(a, b, c, d): Addr) -> [u8, ..4] {
  [a, b, c, d]
}



// key:    adjacent ip (next hop)
// value:  index to InterfaceRow (see below)
pub type InterfaceTable = HashMap<Addr, uint>;

pub struct InterfaceRow {
  pub local_ip:  Addr,
  pub interface: RWLock<Box<dl::Interface + Send + Sync + 'static>>,
}

// TODO: use Box<[u8]> instead of Vec<u8>
// TODO: real network card may consolidate multiple packets per interrupt
pub type Handler = //Handler<Ip>;
  Box<Fn<(packet::V,), ()> + Send + Sync + 'static>;

pub type ProtocolTable = Vec<Vec<Handler>>;

pub struct State<A> where A: RoutingTable {
  pub interfaces:        Vec<InterfaceRow>,
  pub neighbors:         InterfaceTable,
  pub routes:            A,
  pub protocol_handlers: RWLock<ProtocolTable>,
  // Identification counter? increased with each packet sent out,
  // used in Identification header for fragmentation purposes
}

impl<RT> State<RT> where RT: RoutingTable
{
  pub fn new(interfaces: Vec<InterfaceRow>, neighbors: InterfaceTable) -> Arc<State<RT>>
  {
    let routes: RT = strategy::init_hack::<RT, _>(neighbors.keys().map(|x| *x));

    let state: Arc<State<RT>> = Arc::new(State {
      routes:            routes,
      neighbors:         neighbors,
      interfaces:        interfaces,
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

    for &InterfaceRow { ref interface, .. } in state.interfaces.iter() {
      use self::receive::make_receive_callback;
      (*interface.write())
        .update_recv_handler(make_receive_callback::<RT>(state.clone()));
    }

    strategy::monitor_hack::<RT>(state.clone());

    state
  }

  /// Returns dl::Interface struct for the requested interface
  pub fn get_interface<'a> (&'a self, interface_ix: uint) -> Option<&'a InterfaceRow>
  {
    self.interfaces.as_slice().get(interface_ix)
  }
}
