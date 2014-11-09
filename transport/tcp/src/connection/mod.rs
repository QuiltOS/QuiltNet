use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::io::net::ip::Port;
use std::sync::{
  RWLock,
  RWLockReadGuard,
  RWLockWriteGuard,
};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

pub mod state;

pub mod syn_sent;
pub mod established;

pub enum Connection {
  Closed,
  SynSent(syn_sent::SynSent),
  //SynReceived(syn_received::SynReceived),
  Established(established::Established),
}

/// Guaranteed to be closed to safe to mutate
pub fn reserve
  <'state, A, R>
  (state:     &'state ::State<A>,
   us:        Port,
   them:      ::ConAddr,
   k:         |&mut Connection| -> Result<R, ()>)
   -> Result<R, ()>
  where A: RoutingTable
{
  let lock0 = state.tcp.read();

  let conns = match lock0.get(&us) {
    None    => panic!("but.. but... we exist!"),
    Some(p) => &p.connections,
  };
  let mut lock1 = conns.write();

  //  Ok(lock0) // ((lock0, lock1))
  let conn = match lock1.entry(them) {
    Vacant(entry)   => entry.set(RWLock::new(Closed)),
    Occupied(entry) => entry.into_mut(),
  };

  //lock.downgrade(); // TODO: get us a read lock instead
  let mut lock2 = conn.write();

  match *lock2 {
    Closed => (),
    _      => return Err(()),
  };

  k(&mut *lock2)
}
