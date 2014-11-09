//#![feature(unboxed_closures)]
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

use std::collections::HashMap;
use std::io::net::ip::Port;
use std::sync::{Arc, RWLock};

use time::{Timespec, get_time};

use network::ipv4;
use network::ipv4::strategy::RoutingTable;

use listener::Listener;
use connection::Connection;


mod packet;
mod ringbuf;
mod access;

mod send;
mod receive;

mod listener;
mod connection;

pub mod capability;


const PROTOCOL: u8 = 6;

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

pub type Table = HashMap<Port, PerPort>;

pub struct State<A> where A: RoutingTable {
  tcp: RWLock<Table>,
  ip:  Arc<ipv4::State<A>>,
}

pub type SubTable = HashMap<ConAddr, RWLock<Connection>>;

pub struct PerPort {
  listener:    RWLock<Listener>,
  connections: RWLock<SubTable>,
}
