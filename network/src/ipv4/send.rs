use std::result;
use std::error::FromError;

use super::{
  packet,
  strategy,
};

use data_link::interface as dl;


#[deriving(PartialEq, Eq, Clone, Show)]
pub enum Error {
  NoRoute,
  BadPacket(packet::BadPacket),
  External(dl::Error),
}

pub type Result<T> = ::std::result::Result<T, self::Error>;


pub fn send
  <'st, 'clos, A, E>
  (state:              &'st super::State<A>,
   dst:                super::Addr,
   protocol:           u8,
   expected_body_size: Option<u16>,
   builder:            <'a> |&'a mut packet::V|:'clos -> result::Result<(), E>,
   // TODO: make this take a &'a mut packet::A someday
   awkward:            <'a> |&'a mut packet::V|:'clos -> result::Result<(), E>)
   -> result::Result<(), E>
  where A: strategy::RoutingTable,
        E: FromError<self::Error>
{
  let closure: <'p> |&'p mut packet::V| -> result::Result<&'st super::InterfaceRow, E>
    = |packet| {
      try!(builder(packet));
      debug!("client built packet: {}", packet);

      let row = try!(resolve_route(state, dst));
      packet.borrow_mut().set_source(row.local_ip);

      // TCP needs to hook in here for checksum of "virtual header"
      // awkward layer violation is awkward
      try!(awkward(packet));

      Ok(row)
    };

  let (row, packet) = try!(packet::V::new_with_builder(
    dst,
    protocol,
    expected_body_size,
    closure));

  // final try to do from_error
  try!(send_manual(row, packet).map_err(self::External));
  Ok(())
}


/// looks up route for interface and packet src address
pub fn resolve_route<'st, A>(state: &'st super::State<A>,
                             dst:   super::Addr)
                             -> self::Result<&'st super::InterfaceRow>
  where A: strategy::RoutingTable
{
  match state.routes.lookup(dst) {
    None           => Err(self::NoRoute),
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
pub fn send_manual(
  row:            &super::InterfaceRow,
  packet:         packet::V)
  -> dl::Result<()>
{
  let &super::InterfaceRow { ref interface, .. } = row;
  // need to let here because send consumes packet
  let dst = packet.borrow().get_destination();
  try!(interface.write().send(packet.to_vec()));
  debug!("sent packet to {}", dst);
  Ok(())
}
