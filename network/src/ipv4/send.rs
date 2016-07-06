use std::result;
use std::convert::From;

use super::{
  packet,
  strategy,
};

use data_link::interface as dl;


#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Error<E> {
  NoRoute,
  BadPacket(packet::BadPacket),
  External(dl::Error<E>),
}

impl<E> From<dl::Error<E>> for Error<E> {
  fn from(e: dl::Error<E>) -> Error<E> {
    Error::External(e)
  }
}

pub type Result<T, E> = ::core::result::Result<T, self::Error<E>>;


pub fn send
  <'st, A, DE, E, F, G>
  (state:              &'st super::State<A, DE>,
   dst:                super::Addr,
   protocol:           u8,
   expected_body_size: Option<u16>,
   builder:            F,
   // TODO: make this take a &'st mut packet::A someday
   awkward:            G)
   -> result::Result<(), E>
  where A: strategy::RoutingTable,
        E: From<self::Error<DE>>,
        F: for<'b> FnOnce(&'b mut packet::V) -> result::Result<(), E> + 'st,
        G: for<'b> FnOnce(&'b mut packet::V) -> result::Result<(), E> + 'st,
{
  let closure //: for<'p> |&'p mut packet::V| ->
    = move |packet: &mut packet::V| -> result::Result<&'st super::InterfaceRow<DE>, E> {
      try!(builder(packet));
      debug!("client built packet: {}", packet);

      let row = try!(resolve_route(state, dst));
      packet.borrow_mut().set_source(row.local_ip);

      // TCP needs to hook in here for checksum of "virtual header"
      // awkward layer violation is awkward
      try!(awkward(packet));

      Ok(row)
    };

  let (row, packet) = try!(packet::V::new_with_builder::<E, &'st super::InterfaceRow<DE>, _>(
    dst,
    protocol,
    expected_body_size,
    closure));

  // final try to do from_error
  try!(send_manual(row, packet).map_err(|e| Error::External(e)));
  Ok(())
}


/// looks up route for interface and packet src address
pub fn resolve_route<'st, A, E>(state: &'st super::State<A, E>,
                                dst:   super::Addr)
                               -> self::Result<&'st super::InterfaceRow<E>, E>
  where A: strategy::RoutingTable
{
  match state.routes.lookup(dst) {
    None           => Err(self::Error::NoRoute),
    Some(next_hop) => { // Send packet to next hop towards destination
      debug!("Found route through {}", next_hop);
      match state.neighbors.get(&next_hop) {
        None => panic!("IP: Route's next hop is not a neighbor!"),
        // Tell interface to send packet bytes
        Some(index) => Ok(&state.interfaces[*index])
      }
    }
  }
}


/// For anybody that wants to do their own routing
/// and set their own checksum
pub fn send_manual<E>(
  row:            &super::InterfaceRow<E>,
  packet:         packet::V)
  -> dl::Result<(), E>
{
  let &super::InterfaceRow { ref interface, .. } = row;
  // need to let here because send consumes packet
  let dst = packet.borrow().get_destination();
  try!(interface.write().unwrap().send(packet.to_vec()));
  debug!("sent packet to {}", dst);
  Ok(())
}
