use std::collections::HashMap;
use std::collections::hash_map::{Occupied, Vacant};
use std::io::net::ip::Port;
use std::sync::{
  RWLock,
  RWLockReadGuard,
  RWLockWriteGuard,
};

use listener::Listener;
use connection::Connection;

use send;

#[inline]
pub fn get_per_port
  <'s>
  (tcp_table:  &'s RWLockReadGuard<'s, ::Table>,
   local_port: Port)
   -> Result<&'s::PerPort, ()>
{
  match tcp_table.get(&local_port) {
    Some(st) => Ok(st),
    None     => Err(())
  }
}

#[inline]
pub fn reserve_per_port_mut
  <'s>
  (tcp_table:  &'s mut RWLockWriteGuard<'s, ::Table>,
   local_port: Port)
   -> &'s mut ::PerPort
{
  match tcp_table.entry(local_port) {
    Vacant(entry)   => entry.set(::PerPort { // allocate blank
      listener:    RWLock::new(Listener::Closed),
      connections: RWLock::new(HashMap::new()),
    }),
    Occupied(entry) => entry.into_mut(),
  }
}

#[inline]
pub fn reserve_connection_mut
  <'s>
  (subtable:     &'s mut RWLockWriteGuard<'s, ::SubTable>,
   foreign_addr: ::ConAddr)
   -> &'s mut RWLock<Connection>
{
  match subtable.entry(foreign_addr) {
    Vacant(entry)   => entry.set(RWLock::new(Connection::Closed)),
    Occupied(entry) => entry.into_mut(),
  }
}
