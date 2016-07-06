use core::fmt::Debug;
use std::sync::Arc;

use super::{
  packet,
  strategy,
  send
};

use data_link::interface as dl;

/// Called upon receipt of an IP packet:
/// If packet is destined for this node, deliver it to appropriate handlers
/// If packet is destined elsewhere, fix packet headers and forward
fn receive<'a, A, E>(state: &super::State<'a, A, E>, buf: Vec<u8>)
  where A: strategy::RoutingTable<'a> + 'a,
        E: Debug
{
  debug!("Received packet.");
  let packet = match packet::validate(buf.as_slice()) {
    Ok(_)  => packet::V::new(buf),
    Err(e) => {
      debug!("dropping incomming packet because {:?}", e);
      return;
    },
  };

  debug!("packet header:\n{}", packet.borrow());

  if is_packet_dst_local(state, &packet) {
    debug!("Packet is local! {}", packet);
    // local handling
    let handlers = &state.protocol_handlers.read().unwrap()
      [packet.borrow().get_protocol() as usize];
    // If there are no handlers (vector is empty), the packet is just dropped
    // TODO: factor out this clone-until-last-time pattern
    let mut iter = handlers.iter().peekable();
    while let Some(ref handler) = iter.next() {
      if iter.peek().is_none() {
        handler(packet);
        break;
      } else {
        handler(packet.clone());
      }
    }
    /*
    match iter.next() {
      None => (),
      Some(mut handler) => loop {
        match iter.next() {
          None         => {
            // no clone needed!
            (&**handler).call((packet,));
            break;
          }
          Some(h_next) => {
            (&**handler).call((packet.clone(),));
            handler = h_next;
          }
        }
      }
    }*/
  } else {
    debug!("packet is not local! {}", packet);
    // handle errors just for logging purposes
    match forward(state, packet) {
      Ok(_) => (),
      Err(e) => debug!("packet could not be fowarded because {:?}", e),
    };
  }
}

/// Forwards a packet back into the network after rewriting its headers
/// Result status is whether packet was able to be forwarded
fn forward<'a, A, E>(state: &super::State<'a, A, E>, mut packet: packet::V) -> send::Result<(), E>
  where A: strategy::RoutingTable<'a> + 'a
{
  { // Decrement TTL
    let ttl = packet.borrow().get_time_to_live() - 1;
    if ttl == 0 { return Ok(()); }
    packet.borrow_mut().set_time_to_live(ttl);
  }
  { // Update checksum
    packet.borrow_mut().update_checksum();
  }
  let row = try!(send::resolve_route(
    state,
    packet.borrow_mut().get_destination()));
  // Do NOT update src address
  try!(send::send_manual(row, packet));
  Ok(())
}

/// Determine whether packet is destined for this node
fn is_packet_dst_local<'a, A, E>(state: &super::State<'a, A, E>, packet: &packet::V) -> bool
  where A: strategy::RoutingTable<'a> + 'a
{
  let dst = packet.borrow().get_destination();

  // TODO: factor out is_neighbor_addr and is_our_addr
  state.interfaces.iter()
    .any(|&super::InterfaceRow { local_ip, .. }| local_ip == dst)
}

pub fn make_receive_callback<'a, A, E>(state: Arc<super::State<'a, A, E>>) -> dl::Handler
  where A: strategy::RoutingTable<'a> + Send + 'a,
        E: Debug + 'a
{
  let state = state.clone();
  box move |packet: dl::Packet | {
    receive(&*state, packet);
  }
}
