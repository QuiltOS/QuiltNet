use std::sync::Arc;

use super::packet2 as packet;

use misc::interface::{MyFn, Handler};

use data_link::interface as dl;

use ipv4::{strategy, send};
use ipv4::{InterfaceRow, IpState};


/// Called upon receipt of an IP packet:
/// If packet is destined for this node, deliver it to appropriate handlers
/// If packet is destined elsewhere, fix packet headers and forward
fn receive<A>(state: &IpState<A>, buf: Vec<u8>)
  where A: strategy::RoutingTable
{
  let packet = match packet::validate(buf.as_slice()) {
    Ok(_)  => packet::V::new(buf),
    Err(e) => {
      debug!("dropping incomming packet because {}", e);
      return;
    },
  };

  if is_packet_dst_local(state, &packet) {
    debug!("Packet is local! {}", packet);
    // local handling
    let handlers = &(*state.protocol_handlers.read())
      [packet.borrow().get_protocol() as uint];
    // If there are no handlers (vector is empty), the packet is just dropped
    // TODO: copy packet only if there are multiple handlers
    for handler in handlers.iter() {
      (&**handler).call((packet.clone(),));
    }
  } else {
    debug!("packet is not local! {}", packet);
    // handle errors just for logging purposes
    match forward(state, packet) {
      Ok(_) => (),
      Err(e) => debug!("packet could not be fowarded because {}", e),
    };
  }
}

/// Forwards a packet back into the network after rewriting its headers
/// Result status is whether packet was able to be forwarded
fn forward<A>(state: &IpState<A>, mut packet: packet::V) -> send::Result<()>
  where A: strategy::RoutingTable
{
  { // decrement TTL
    let ttl = packet.borrow().get_time_to_live() - 1;
    if ttl == 0 { return Ok(()); }
    packet.borrow_mut().set_time_to_live(ttl);
  }
  { // do something with checksum ?

  }
  //// map Error because Fix_headers does not return IoError
  //try!(fix_headers(&mut packet).map_err(|_| ::std::io::IoError {
  //  kind:   ::std::io::InvalidInput,
  //  desc:   "Packet had invalid headers",
  //  detail: None,
  //}));
  send::send(state, packet)
}

/// Determine whether packet is destined for this node
fn is_packet_dst_local<A>(state: &IpState<A>, packet: &packet::V) -> bool
  where A: strategy::RoutingTable
{
  let dst = packet.borrow().get_destination();

  // TODO: factor out is_neighbor_addr and is_our_addr
  state.interfaces.iter()
    .any(|&InterfaceRow { local_ip, .. }| local_ip == dst)
}

struct IpDl<A>
  where A: strategy::RoutingTable + Send
{
  state: Arc<IpState<A>>,
}

impl<A> MyFn<(dl::Packet,), ()> for IpDl<A>
  where A: strategy::RoutingTable + Send
{
  fn call(&self, args: (dl::Packet,)) {
    let (packet,) = args;
    receive(&*self.state, packet);
  }
}

pub fn make_receive_callback<A>(state: Arc<IpState<A>>) -> dl::Handler
  where A: strategy::RoutingTable + Send
{
  box IpDl { state: state.clone() }
}
