use std::io::{IoError, IoResult, IoUnavailable, NotConnected};
use std::io::net::ip::{Ipv4Addr, IpAddr};

use packet::ipv4 as packet;

use network::ipv4::{strategy, IpState, InterfaceRow};
//use network::ipv4::packet::{IpAddr, IpPacket};

//TODO: visibility?
pub fn send_with_client<A>(state:              &IpState<A>,
                           ip:                 IpAddr,
                           protocol:           u8,
                           expected_body_size: Option<u16>,
                           client:             |&mut packet::V| -> IoResult<()>)
                           -> IoResult<()>
  where A: strategy::RoutingTable
{
  let p = try!(packet::V::new_with_client(ip,
                                          protocol,
                                          expected_body_size,
                                          client));
  println!("built packet: {}", p);
  try!(send(state, p));
  println!("sent");
  Ok(())
}

const NO_ROUTE_ERROR: IoError = IoError {
  kind: NotConnected,
  desc: "No routing table entry for this packet",
  detail: None,
};

//TODO: visibility?
//TODO: move, not copy, packet for final interface
pub fn send<A>(state: &IpState<A>, packet: packet::V) -> IoResult<()>
  where A: strategy::RoutingTable
{
  match packet.borrow().get_destination() {
    //  The following is broken:
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
    _ => match state.routes.lookup(packet.borrow().get_destination()) {
      None => (), // drop, no route to destination

      // Send packet to next hop towards destination
      // TODO: include loopback address in routing table
      // TODO: include broadcast interface w/ overloaded send fn
      Some(next_hop) => {
        println!("Found route through {}", next_hop);
        match state.ip_to_interface.find(&next_hop) {
          // drop, next hop isn't in our interface map
          None => return Err(NO_ROUTE_ERROR.clone()),
          // Tell interface to send packet bytes
          Some(index) => {
            try!(send_manual(packet, &state.interfaces[*index]));
          }
        }
      }
    }
  }
  Ok(())
}

// Public for rip, or anybody that wants to do their own routing
pub fn send_manual(mut packet: packet::V, row: &InterfaceRow) -> IoResult<()> {
  let &InterfaceRow { local_ip, ref interface } = row;
  let _ = packet.borrow_mut().set_source(local_ip); // ip for this interface
  interface.write().send(packet.to_vec())
}
