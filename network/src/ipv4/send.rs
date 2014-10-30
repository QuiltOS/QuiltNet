use std::io::net::ip::IpAddr;

use super::packet2 as packet;

use ipv4::{strategy, IpState, InterfaceRow};
//use ipv4::packet::{IpAddr, IpPacket};

use data_link::interface as dl;

//TODO: visibility?
pub fn send_with_client<A>(state:              &IpState<A>,
                           ip:                 IpAddr,
                           protocol:           u8,
                           expected_body_size: Option<u16>,
                           client:             |&mut packet::V| -> self::Result<()>)
                           -> self::Result<()>
  where A: strategy::RoutingTable
{
  let p = try!(packet::V::new_with_client(ip,
                                          protocol,
                                          expected_body_size,
                                          client));
  println!("IP: built packet: {}", p);
  send(state, p)
}

#[deriving(PartialEq, Eq, Clone, Show)]
pub enum Error {
  NoRoute,
  BadPacket(packet::BadPacket),
  External(dl::Error),
}

pub type Result<T> = ::std::result::Result<T, self::Error>;

pub fn send<A>(state: &IpState<A>, packet: packet::V) -> self::Result<()>
  where A: strategy::RoutingTable
{
  //if state.neighbors.contains_key(&packet.borrow().get_destination()) {
  //  return Err(::std::io::IoError {
  //    kind:   ::std::io::InvalidInput,
  //    desc:   "Cannot send IP to ourself!",
  //    detail: None,
  //  })
  //}

  //  TODO: The following is broken:
  //  // broadcast,
  //  Ipv4Addr(0,0,0,0) =>
  //    for row in state.interfaces.iter() {
  //      try!(send_manual(packet.clone(), row));
  //    },
  //  // neighbor cast
  //  Ipv4Addr(0,0,0,1) =>
  //    for row in state.interfaces.iter() {
  //      let &(_, dest, _) = row;
  //      let _ = packet.borrow_mut().set_destination(dest);
  //      try!(send_manual(packet.clone(), row));
  //    },

  let dst = packet.borrow().get_destination();
  match state.routes.lookup(dst) {
    None           => Err(self::NoRoute),
    Some(next_hop) => { // Send packet to next hop towards destination
      println!("IP: Found route through {}", next_hop);
      match state.neighbors.find(&next_hop) {
        None => fail!("IP: Route's next hop is not a neighbor!"),
        // Tell interface to send packet bytes
        Some(index) => {
          send_manual(packet, &state.interfaces[*index]).map_err(self::External)
        }
      }
    }
  }
}

// Public for rip, or anybody that wants to do their own routing
pub fn send_manual(mut packet: packet::V, row: &InterfaceRow) -> dl::Result<()> {
  let &InterfaceRow { local_ip, ref interface } = row;
  let dst = packet.borrow().get_destination();
  let _   = packet.borrow_mut().set_source(local_ip); // ip for this interface
  try!(interface.write().send(packet.to_vec()));
  println!("IP: sent packet to {}", dst);
  Ok(())
}
