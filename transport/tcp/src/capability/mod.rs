/// A TCP Capability is mostly analogous to a (TCP) Socket on
/// Unix. This library wraps the more low-level core implementation to
/// provide a "blocking", synchronous interface.
///
/// While in the core interface connections and listeners need to be
/// explicitly closed, that would be undesirable burden on the users
/// of this implementation. Once all capabilities are dropped, nobody
/// will be able to access the underlying connection without using the
/// core interface. Therefore, once they are all dropped the
/// connection will be automatically closed. If you don't like that,
/// use the core interface.


// TODO: clonable capabilities


pub use self::l::L;
pub use self::c::C;

mod l;
mod c;
