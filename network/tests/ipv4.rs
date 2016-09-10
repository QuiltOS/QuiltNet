#![feature(box_syntax)]

mod net {
  pub extern crate misc;

  pub mod data_link {
    pub extern crate interface;
    pub extern crate udp_mock;
  }

  pub extern crate network;

  pub mod transport {
    //pub extern crate brown_rip;
    pub extern crate static_routing;
  }
}

use std::fmt;
use std::str::from_utf8;
use std::sync::{Arc, Barrier, RwLock};

#[macro_use]
extern crate log;

use net::data_link::udp_mock::*;
use net::network::ipv4;
use net::network::ipv4::*;
use net::network::ipv4::strategy::RoutingTable;
use net::transport::static_routing::StaticTable;

pub fn make_ip_to_wait
  <'st, R, E>
  (interfaces: Vec<InterfaceRow<'st, E>>,
   neighbors: InterfaceTable,
   msg: &'st str,
   barrier: Arc<Barrier>)
   -> Arc<State<'st, R, E>>
  where R: strategy::RoutingTable<'st> + 'st,
        E: fmt::Debug + 'st

{
  let state = ipv4::State::<'st, R, E>::new(interfaces, neighbors);
  control::register_protocol_handler::<R, E>(&*state, 8, box move |packet| {
    debug!("got packet: {}", from_utf8(packet.borrow().get_payload()).unwrap());
    debug!("matching against: {}", msg);
    assert_eq!(packet.borrow().get_payload(), msg.as_bytes());
    barrier.wait();
  });
  state
}

fn sending
  <'st, R, E>
  (state: &ipv4::State<'st, R, E>,
   dst:   ipv4::Addr,
   msg:   &'static str)
   -> Result<(), ipv4::send::Error<E>>
  where R: RoutingTable<'st>,
        E: 'st
{
  let buf: &[u8] = msg.as_bytes();
  send::send(
    state,
    dst,
    8,
    Some(buf.len() as u16),
    | packet | {
      packet.as_mut_vec().extend_from_slice(buf);
      Ok(())
    },
    |_| Ok(()) )
}

macro_rules! map(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

#[test]
fn direct_two_nodes() {
  static NUM_THREADS: usize = 1;

  let barrier = Arc::new(Barrier::new(3));

  let (l1, da1) = Listener::new_loopback(NUM_THREADS).unwrap();
  let (l2, da2) = Listener::new_loopback(NUM_THREADS).unwrap();

  let di1 = Interface::new(&l1, da2, box |_|());
  let di2 = Interface::new(&l2, da1, box |_|());

  let ia1 = ipv4::Addr([1,1,1,1]);
  let ia2 = ipv4::Addr([2,2,2,2]);

  const M1: &'static str = "Hey Node 1!";
  const M2: &'static str = "Hey Node 2!";

  let i1 = make_ip_to_wait::<StaticTable, _>(
    vec![InterfaceRow { local_ip: ia1, interface: RwLock::new(box di1) }],
    map!{ia2 => 0},
    M1,
    barrier.clone());

  let i2 = make_ip_to_wait::<StaticTable, _>(
    vec![InterfaceRow { local_ip: ia2, interface: RwLock::new(box di2) }],
    map!{ia1 => 0},
    M2,
    barrier.clone());

  sending(&*i1, ia2, M2).unwrap();
  sending(&*i2, ia1, M1).unwrap();

  barrier.wait();
}
