// in the short run these are not important
#![allow(unused_imports)]
#![allow(dead_code)]

#![feature(if_let)]
//#![feature(associated_types)]
#![feature(default_type_params)]
#![feature(unboxed_closures)] // still broken
#![feature(slicing_syntax)]
#![feature(tuple_indexing)]
#![feature(phase)]

// for tests
#![feature(globs)]

#[cfg(not(ndebug))]
#[phase(plugin, link)]
extern crate log;

#[phase(plugin, link)]
extern crate misc;
extern crate time;
extern crate network;

use std::fmt;
use std::collections::HashMap;
use std::default::Default;
use std::io::net::ip::Port;
use std::sync::{Arc, RWLock};

use time::{Timespec, get_time};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use concurrent_hash_map::ConcurrentHashMap;
use listener::Listener;
use connection::Connection;

mod packet;
mod concurrent_hash_map;
mod ring_buf;

mod send;
mod receive;

mod trace;

pub mod listener;
pub mod connection;

pub mod capability;


pub const PROTOCOL: u8 = 6;

/// Address of one end of a connection
pub type ConAddr = (ipv4::Addr, Port);

/// Closed state and memory usage:
///
/// We keep track of closed connections/listeners inside the `Connection` and
/// `Listener` structs, rather than with the existence of such structs. The
/// reason for this is to free /all/ memory associated with a connection
/// requires getting a lock on entire tables to evict the entries -- an
/// synchronization affecting other connections. Putting the `Option` outside
/// the `RWLock` for the listener instills the same semantics for consistency
/// (and similar performance either way).
///
/// The memory associated with connections and listeners shall instead be
/// collected periodically, minimizing the aforementioned write-locking of the
/// whole-map locks.
///
/// A concurrent map that provided RW-synchronization to the /entries/ in the
/// map (as opposed to the values) would enable the presence of an entry to
/// indicate open/closed state of the connection.
///
///
/// Concurrency Model:
///
/// Like the rest of this implementation, callbacks are used in order to give
/// the library user the maximum flexibility over the scheduling model. This
/// means it is important that the tables have `Arc<T>`s and not `Weak<T>`s so
/// that the connection persists between callback invocations.

pub type Table = ConcurrentHashMap<Port, PerPort>;

pub struct State<A> where A: RoutingTable {
  tcp:    Table,
  pub ip: Arc<ipv4::State<A>>, // not TCP's responsibility to hide this
}

impl<A> State<A> where A: RoutingTable
{
  pub fn init_and_register(ip: Arc<ipv4::State<A>>) -> Arc<self::State<A>>
  {
    let ptr = Arc::new(State {
      ip:  ip,
      tcp: ConcurrentHashMap::new(),
    });
    receive::register(&ptr);
    ptr
  }

  pub fn dump_tcp(&self) {
    println!("Sockets:");
    self.tcp.dump();
  }
}

pub type SubTable = ConcurrentHashMap<ConAddr, RWLock<Connection>>;

impl fmt::Show for RWLock<Connection> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "RW<{}>", self.read().deref())
  }
}

impl fmt::Show for RWLock<Option<Listener>> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "RW<{}>", self.read().deref())
  }
}

pub struct PerPort {
  listener:    RWLock<Option<Listener>>,
  connections: SubTable,
}



impl Default for PerPort
{
  fn default() -> PerPort {
    PerPort {
      listener:    RWLock::new(Default::default()),
      connections: ConcurrentHashMap::new(),
    }
  }
}

impl PerPort
{
  pub fn get_or_init(tcp: &Table, us:  Port) -> Arc<PerPort>
  {
    tcp.get_or_init(
      us,
      || Default::default())
  }
}

impl fmt::Show for PerPort {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Listener: {}, Cnx:TODO", self.listener) //, self.connections)
  }
}
