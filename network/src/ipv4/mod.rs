use std::collections::hash_map::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use data_link::interface as dl;

use self::strategy::RoutingTable;

pub mod control;
pub mod packet;
pub mod send;
pub mod receive;
pub mod strategy;


#[derive(PartialEq, PartialOrd, Eq, Ord,
         Clone, Hash, Show)]
pub struct Addr(pub u8, pub u8, pub u8, pub u8);

impl Copy for Addr { }

impl fmt::String for Addr {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}.{}.{}.{}", self.0, self.1, self.2, self.3)
  }
}

impl FromStr for Addr {
  fn from_str(s: &str) -> Option<Addr> {
    let mut quad: [Option<u8>; 4] = [None, None, None, None];
    let iter = s.trim().split('.').map(|x| FromStr::from_str(x));
    
    for (mut ptr, val) in quad.iter_mut().zip(iter)
    {
      *ptr = val;
    }
    match quad {
      [Some(q1), Some(q2), Some(q3), Some(q4)] => Some(Addr(q1, q2, q3, q4)),
      _ => None
    }
  }
}


#[inline]
pub fn parse_addr(&[a, b, c, d]: &[u8; 4]) -> Addr {
  Addr(a, b, c, d)
}

#[inline]
pub fn parse_addr_unsafe(b: &[u8]) -> Addr {
  Addr(b[0], b[1], b[2], b[3])
}

#[inline]
pub fn write_addr(Addr(a, b, c, d): Addr) -> [u8; 4] {
  [a, b, c, d]
}



// key:    adjacent ip (next hop)
// value:  index to InterfaceRow (see below)
pub type InterfaceTable = HashMap<Addr, usize>;

pub struct InterfaceRow {
  pub local_ip:  Addr,
  pub interface: RwLock<Box<dl::Interface + Send + Sync + 'static>>,
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
  pub protocol_handlers: RwLock<ProtocolTable>,
  // Identification counter? increased with each packet sent out,
  // used in Identification header for fragmentation purposes
}

impl<RT> State<RT> where RT: RoutingTable
{
  pub fn new(interfaces: Vec<InterfaceRow>, neighbors: InterfaceTable) -> Arc<State<RT>>
  {
    let routes: RT = RoutingTable::init(neighbors.keys().map(|&: x| *x));

    let state: Arc<State<RT>> = Arc::new(State {
      routes:            routes,
      neighbors:         neighbors,
      interfaces:        interfaces,
      // handlers are not clonable, so the nice ways of doing this do not work
      protocol_handlers: RwLock::new(vec!(
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
      interface.write().unwrap()
        .update_recv_handler(make_receive_callback::<RT>(state.clone()));
    }

    RoutingTable::monitor(state.clone());

    state
  }

  /// Returns dl::Interface struct for the requested interface
  pub fn get_interface<'a> (&'a self, interface_ix: usize) -> Option<&'a InterfaceRow>
  {
    self.interfaces.as_slice().get(interface_ix)
  }
}
