use std::io::net::ip::{IpAddr, Ipv4Addr};
use std::mem::transmute;
use std::sync::Arc;

use network::ipv4::{strategy, control, IpState, IpHandler, InterfaceRow};
use network::ipv4::send::send_manual;

use packet::ipv4::V as Ip;

use interface::MyFn;

use super::{
  RipTable,
  RipRow,
};
use super::packet;

struct RipHandler { _state: Arc<IpState<RipTable>> }

impl MyFn<(Ip,), ()> for RipHandler {

  fn call(&self, _args: (Ip,)) {
  }
}


/// Runs simple debug handler, printing out all packets received for the given protocols
pub fn register(state: Arc<IpState<RipTable>>) {
  control::register_protocol_handler(
    &*state,
    200,
    box RipHandler { _state: state.clone() })
}


/// This method is the sole method of sending a "response" packet.
///
/// The `key_rows` are written to packets, one per each interface / neighbor. Entries learned about
/// from the neighbor in question will be "poisoned" accordingly. This is fine for the case of
/// sending expired packets to other nodes, as the cost field would be infinite anyways.
///
/// Note that unlike the normal send method, this does not take any locks
pub fn propagate<'a, I, J>(key_rows: || -> I, mut interfaces: J)
  where I: Iterator<(IpAddr, &'a RipRow)>,
        J: Iterator<&'a InterfaceRow>
{
  for interface_row in interfaces {
    let &(_, ref dst, ref interface) = interface_row;

    let entries_iter = key_rows().map(|(ip, row)| packet::Entry {
      address: match ip {
        Ipv4Addr(a, b, c, d) => unsafe { transmute([a,b,c,d]) },
        _                    => fail!("no IPv6 implemented yet")
      },
      cost: if row.learned_from == dst.clone() {
        //poison
        16
      } else {
        row.cost as u32
      }
    });

    let packet_thunk = packet::write(packet::Response(entries_iter));

  }
}
