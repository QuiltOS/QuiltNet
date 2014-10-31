use std::io::IoResult;
use std::io::net::ip::IpAddr;
use std::sync::Arc;
use std::option::None;

use network::ipv4::{
  IpState,
  InterfaceRow,
  InterfaceTable,
};
use network::ipv4::control;
use network::ipv4::send::send_manual;

use network::ipv4::packet2::V as Ip;

use misc::interface::MyFn;

use super::{RIP_INFINITY, RipTable, RipRow};
use super::packet::{mod, Request, Response};

struct RipHandler { state: Arc<IpState<RipTable>> }

impl MyFn<(Ip,), ()> for RipHandler {

  fn call(&self, (packet,):(Ip,)) {
    handle(&*self.state, packet).unwrap(/* "Failure handling incomming IP Packet" */);
  }

}

fn handle(state: &IpState<RipTable>, packet: Ip) -> IoResult<()> {
  let neighbor_addr = packet.borrow().get_source();
  //let interface_addr = packet.borrow().get_destination();
  let data = packet.borrow().get_payload();

  match state.neighbors.find(&neighbor_addr) {
    None    => debug!("RIP: Odd, got packet from non-neighbor: {}", neighbor_addr),
    _       => (),
  };

  match packet::parse(data) {
    Ok(Request) => {
      debug!("RIP: Got request from {}", neighbor_addr);

      // TODO: factor out singleton iterator

      let single = [neighbor_addr];
      let unlocked = state.routes.map.write();
      let factory = || unlocked.iter().map(|(a,r)| (*a,r)); // the whole table

      try!(propagate(factory,
                     single.iter().map(|x| *x), // just who issued the request
                     &state.neighbors,
                     state.interfaces.as_slice()));

      // TODO factor out empty iterator
      let empty: [packet::Entry, ..0] = [];
      let empty_iter = empty.as_slice().iter().map(|x| *x);

      try!(update(state, neighbor_addr, empty_iter));
    },
    Ok(Response(entries)) => {
      debug!("RIP: Got response from {}", neighbor_addr);
      try!(update(state, neighbor_addr, entries));
    },
    _ => debug!("RIP: invalid packet received: {}", data),
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
  where I: Iterator<(IpAddr, &'a RipRow)>,
        J: Iterator<IpAddr>
{
  for neighbor_ip in neighbor_subset
  {
    let interface_row = match neighbors.find(&neighbor_ip) {
      None         => fail!("Can't propagate to non-neighbor: {}", neighbor_ip),
      Some(&index) => &interfaces[index],
    };

    let entry_builder = |(route_dst, row): (IpAddr, &'a RipRow)| packet::Entry {
      address: route_dst,
      cost: if row.next_hop == neighbor_ip {
        //poison
        RIP_INFINITY
      } else {
        row.cost
      } as u32
    };

    let mut entries_iter = route_subset().map(entry_builder).peekable();

    while !entries_iter.is_empty() {

      let f = |packet: &mut Ip| {
        packet::write_response(&mut entries_iter)(packet.as_mut_vec())
      };

      let packet = try!(Ip::new_with_client(
        neighbor_ip,
        super::RIP_PROTOCOL,
        None,
        f));
      match send_manual(packet, interface_row) {
        Ok(_)  => (),
        Err(e) => debug!("RIP: could not propigate to {}, because {}", neighbor_ip, e),
      };
    }
  }
  Ok(())
}


/// Go through a bunch of entries, update the table, propigate changes
fn update<I>(state: &IpState<RipTable>,
             neighbor_addr: IpAddr,
             entries_but_neighbor_itself: I)
             -> IoResult<()>
  where I: Iterator<packet::Entry>
{
  // TODO: factor out singleton iterator
  // "cons on" neighbor who just responded
  let scratch = [packet::Entry { cost: 0, address: neighbor_addr }];
  let mut entries = scratch.as_slice().iter().map(|x| *x).chain(entries_but_neighbor_itself);

  let mut updated_entries = ::std::collections::hashmap::HashMap::new();

  for packet::Entry { mut cost, address: dst } in entries {
    use std::collections::hashmap::{Occupied, Vacant};

    // hmm, thoughput or latency?
    let mut unlocked = state.routes.map.write();

    if cost > RIP_INFINITY as u32 {
      debug!("RIP: received bad cost grater than infinity: {}", cost);
    };
    if cost < RIP_INFINITY as u32 { cost += 1; }; // bump cost unless infinite

    debug!("RIP: can go to {} with cost {} via {}", dst, cost, neighbor_addr);

    let mk_new_row = || {
      RipRow {
        time_added: ::time::get_time(),
        next_hop: neighbor_addr,
        cost: cost as u8,
      }
    };

    match unlocked.entry(dst) {
      Vacant(entry) => {
        if cost < RIP_INFINITY as u32 { // no point spending memory on dead routes
          let r = mk_new_row();
          updated_entries.insert(dst, r);
          entry.set(r.clone());
        }
      },
      Occupied(e) => {
        let old = e.into_mut();

        let no_worse   = cost           <= old.cost as u32;
        let update     = neighbor_addr == old.next_hop;
        let dead_route = cost           >= RIP_INFINITY as u32;
        let to_self    = state.interfaces.iter()
          .any(|&InterfaceRow { local_ip, .. }| local_ip == dst);

        // accept update from neighbor, or better route
        // don't bother switching what sort of dead route it is
        // don't bother accepting route to self
        if (update || no_worse) && !dead_route && !to_self
        {
          let new = mk_new_row();
          debug!("RIP: route to {} changed from ({}, {}) to ({}, {})",
                   dst, old.cost, old.next_hop, new.cost, new.next_hop);

          // only propigate updates that effect cost
          // nobody cares about our next hop
          // routes renews (i.e. only timestamp changed) are only propigated via periodic updates
          if new.cost != old.cost {
            updated_entries.insert(dst, new);
          }

          *old = new;
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
  Ok(())
}
