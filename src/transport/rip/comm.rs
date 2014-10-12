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

use packet::ipv4::V as Ip;

use interface::MyFn;

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
    None    => println!("RIP: Odd, got packet from non-neighboor"),
    _       => (),
  };


  match packet::parse(data) {

    Ok(Request) => {
      println!("RIP: Got request from {}", neighboor_addr);

      let single = [neighboor_addr];
      let unlocked = state.routes.map.write();
      let factory = || unlocked.iter().map(|(a,r)| (*a,r));

      try!(propagate(factory,
                     single.iter().map(|x| *x),
                     &state.neighbors,
                     state.interfaces.as_slice()));
    },

    Ok(Response(entries)) => {
      println!("RIP: Got response from {}", neighboor_addr);
      // hmm, thoughput or latency?
      let mut unlocked = state.routes.map.write();

      let mut changed_keys = ::std::collections::HashSet::new();

      for &packet::Entry { cost, address } in entries.iter() {
        use std::collections::hashmap::{Occupied, Vacant};

        let cost = cost + 1; // bump cost

        let dst = packet::parse_ip(address);

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
            entry.set(mk_new_row());

            changed_keys.insert(dst);
          },
          Occupied(e) => {
            let row = e.into_mut();
            let &RipRow { cost: old_cost, .. } = row;
            if old_cost > cost as u8 {
              println!("RIP: route to {} upgraded from {} to {}",
                       neighboor_addr, old_cost, cost);
              *row = mk_new_row();

              changed_keys.insert(dst);
            }
          },
        };
      };

      let factory = || unlocked.iter().map(|(a,r)| (*a,r));

      try!(propagate(factory,
                     changed_keys.iter().map(|x| *x),
                     &state.neighbors,
                     state.interfaces.as_slice()));
    },

    _ => println!("RIP: invalid packet received, oh well..."),

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
pub fn propagate<'a, I, J>(route_subset:        || -> I,
                           mut neighbor_subset: J,
                           neighbors:     &'a InterfaceTable,
                           interfaces:          &'a [InterfaceRow])
                           -> IoResult<()>
  where I: Iterator<(IpAddr, &'a RipRow)>,
        J: Iterator<IpAddr>
{
  for neighbor_ip in neighbor_subset {

    let interface_row = match neighbors.find(&neighbor_ip) {
      None         => fail!("Can't propagate to non-neighbor"),
      Some(&index) => &interfaces[index],
    };

    let packet = try!(Ip::new_with_client(
      neighbor_ip,
      super::RIP_PROTOCOL,
      None,
      |packet| -> IoResult<()> {

        let entry_builder = |(ip, row): (IpAddr, &'a RipRow)| packet::Entry {
          address: packet::write_ip(ip),
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
