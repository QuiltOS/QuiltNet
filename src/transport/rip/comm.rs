use std::io::IoResult;
use std::io::net::ip::{IpAddr, Ipv4Addr};
use std::mem::transmute;
use std::sync::Arc;
use std::option::{Option, None};

use network::ipv4::{strategy, control, IpState, IpHandler, InterfaceRow};
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

  match packet::parse(data) {

    Ok(Request) => {
      match state.ip_to_interface.find(&neighboor_addr) {
        None        => println!("Odd, got RIP packet from non-neighboor"),
        Some(&index) => {
          println!("Got rip request");
          let single = state.interfaces.as_slice()[index..index+1];
          assert!(single.len() == 1);

          let unlocked = state.routes.map.write();
          let factory = || unlocked.iter().map(|(a,r)| (*a,r));

          try!(propagate(factory,
                         single.iter()));
        },
      }
    },

    Ok(Response(entries)) => {
      println!("Got rip response");
      for &packet::Entry { cost, address } in entries.iter() {
        use std::collections::hashmap::{Occupied, Vacant};
        
        // hmm, thoughput or latency?
        let mut unlocked = state.routes.map.write();

        let dst = packet::parse_ip(address);

        let mk_new_row = || {
          use transport::static_routing::StaticRow;
          RipRow {
            time_added: ::time::get_time(),
            rest: StaticRow { next_hop: neighboor_addr },
            cost: cost as u8,
            learned_from: neighboor_addr,
          }
        };
        
        match unlocked.entry(dst) {
          Vacant(entry) => {
            entry.set(mk_new_row());
          },
          Occupied(e) => {
            let row = e.into_mut();
            let &RipRow { cost: old_cost, .. } = row;
            if old_cost > cost as u8 {
              *row = mk_new_row();
            }
          },
        };
      };
    },

    _ => println!("invalid RIP packet received, oh well..."),

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
/// Note that unlike the normal send method, this does not take any locks
pub fn propagate<'a, I, J>(key_rows: || -> I, mut interfaces: J) -> IoResult<()>
  where I: Iterator<(IpAddr, &'a RipRow)>,
        J: Iterator<&'a InterfaceRow>
{
  for interface_row in interfaces {
    let &(_, dst, ref interface) = interface_row;

    let packet = try!(Ip::new_with_client(
      dst,
      super::RIP_PROTOCOL,
      None,
      |packet| -> IoResult<()> {

        let entry_builder = |(ip, row): (IpAddr, &'a RipRow)| packet::Entry {
          address: packet::write_ip(ip),
          cost: if row.learned_from == dst.clone() {
            //poison
            16
          } else {
            row.cost as u32
          }
        };

        let entries_iter = key_rows().map(entry_builder);
        let packet_thunk = packet::write(packet::Response(entries_iter));

        packet_thunk(packet.as_vec())
      }));
    try!(send_manual(packet, interface_row));
  }
  Ok(())
}
