use std::io::IoResult;
use std::io::net::ip::{IpAddr, Ipv4Addr};
use std::mem::transmute;
use std::sync::Arc;
use std::option::{Option, None};

use network::ipv4::{
  IpState,
  IpHandler,
  InterfaceRow,
  InterfaceTable,
};
use network::ipv4::{strategy, control};
use network::ipv4::send::send_manual;

use network::ipv4::packet2::V as Ip;

use misc::interface::MyFn;

use super::{
  RipTable,
  RipRow,
};
use super::packet::{mod, Packet, Request, Response};

struct RipHandler { state: Arc<IpState<RipTable>> }

impl MyFn<(Ip,), ()> for RipHandler {

  fn call(&self, (packet,):(Ip,)) {
    handle(&*self.state, packet).unwrap(/* "Failure handling incomming IP Packet" */);
  }

}

fn handle(state: &IpState<RipTable>, packet: Ip) -> IoResult<()> {
  let neighboor_addr = packet.borrow().get_source();
  //let interface_addr = packet.borrow().get_destination();
  let data = packet.borrow().get_payload();

  match state.neighbors.find(&neighboor_addr) {
    None    => println!("RIP: Odd, got packet from non-neighboor: {}", neighboor_addr),
    _       => (),
  };


  match packet::parse(data) {

    Ok(Request) => {
      println!("RIP: Got request from {}", neighboor_addr);

      let single = [neighboor_addr];
      let unlocked = state.routes.map.write();
      let factory = || unlocked.iter().map(|(a,r)| (*a,r)); // the whole table

      try!(propagate(factory,
                     single.iter().map(|x| *x), // just who issued the request
                     &state.neighbors,
                     state.interfaces.as_slice()));
    },

    Ok(Response(mut entries)) => {
      println!("RIP: Got response from {}", neighboor_addr);

      let mut updated_entries = ::std::collections::hashmap::HashMap::new();

      for packet::Entry { cost, address: dst } in entries {
        use std::collections::hashmap::{Occupied, Vacant};

        // hmm, thoughput or latency?
        let mut unlocked = state.routes.map.write();

        let cost = cost + 1; // bump cost

        println!("RIP: can go to {} with cost {} via {}", dst, cost, neighboor_addr);

        let mk_new_row = || {
          RipRow {
            time_added: ::time::get_time(),
            next_hop: neighboor_addr,
            cost: cost as u8,
          }
        };

        match unlocked.entry(dst) {
          Vacant(entry) => {

            let r = mk_new_row();
            updated_entries.insert(dst, r);

            entry.set(r.clone());
          },
          Occupied(e) => {
            let row = e.into_mut();
            let &RipRow { cost: old_cost, next_hop: old_hop, .. } = row;
            if old_cost >= cost as u8 {
              println!("RIP: route to {} upgraded from ({}, {}) to ({}, {})",
                       dst, old_cost, old_hop, cost, neighboor_addr);

              let r = mk_new_row();
              updated_entries.insert(dst, r);

              *row = r;
            }
          },
        };
      };

      // just those keys which were updated
      let factory = || updated_entries.iter().map(|(a,r)| (*a,r));

      try!(propagate(factory,
                     state.neighbors.keys().map(|x| *x), // tell everyone
                     &state.neighbors,
                     state.interfaces.as_slice()));
    },

    _ => println!("RIP: invalid packet received: {}", data),

  }
  Ok(())
}


/// Runs simple debug handler, printing out all packets received for the given protocols
pub fn register(state: Arc<IpState<RipTable>>) {
  control::register_protocol_handler(
    &*state,
    super::RIP_PROTOCOL,
    box RipHandler { state: state.clone() })
}


/// This method is the sole method of sending a "response" packet.
///
/// The `key_rows` are written to packets, one per each interface / neighbor. Entries learned about
/// from the neighbor in question will be "poisoned" accordingly. This is fine for the case of
/// sending expired packets to other nodes, as the cost field would be infinite anyways.
///
/// Note that unlike the normal send method, this does not take any locks. It purposely asks for
/// certain fields of IpState, rather than the structure as a whole, to prevent itself from taking
/// any locks.
pub fn propagate<'a, I, J>(route_subset:        ||:'a -> I,
                           mut neighbor_subset: J,
                           neighbors:           &'a InterfaceTable,
                           interfaces:          &'a [InterfaceRow])
                           -> IoResult<()>
  where I: Iterator<(IpAddr, &'a RipRow)> + 'a,
        J: Iterator<IpAddr> + 'a
{
  for neighbor_ip in neighbor_subset {

    let interface_row = match neighbors.find(&neighbor_ip) {
      None         => fail!("Can't propagate to non-neighbor: {}", neighbor_ip),
      Some(&index) => &interfaces[index],
    };

    let packet = try!(Ip::new_with_client(
      neighbor_ip,
      super::RIP_PROTOCOL,
      None,
      |packet| -> IoResult<()> {

        let entry_builder = |(route_dst, row): (IpAddr, &'a RipRow)| packet::Entry {
          address: route_dst,
          cost: if row.next_hop == neighbor_ip {
            //poison
            16
          } else {
            row.cost as u32
          }
        };

        let entries_iter = route_subset().map(entry_builder);
        let packet_thunk = packet::write(packet::Response(entries_iter));

        packet_thunk(packet.as_vec())
      }));
    try!(send_manual(packet, interface_row));
  }
  Ok(())
}
