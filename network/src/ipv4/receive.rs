use std::sync::Arc;

use super::{
  packet,
  strategy,
  send
};

use misc::interface::{MyFn, Handler};

use data_link::interface as dl;


/// Called upon receipt of an IP packet:
/// If packet is destined for this node, deliver it to appropriate handlers
/// If packet is destined elsewhere, fix packet headers and forward
fn receive<A>(state: &super::State<A>, buf: Vec<u8>)
  where A: strategy::RoutingTable
{
  debug!("Received packet.");
  let packet = match packet::validate(buf.as_slice()) {
    Ok(_)  => packet::V::new(buf),
    Err(e) => {
      debug!("dropping incomming packet because {}", e);
      return;
    },
  };

  debug!("packet header:");
  // TODO: make print instead return String to write with debug!
  if log_enabled!(::log::DEBUG) { packet.borrow().print(); };

  if is_packet_dst_local(state, &packet) {
    debug!("Packet is local! {}", packet);
    // local handling
    let handlers = &(*state.protocol_handlers.read())
      [packet.borrow().get_protocol() as uint];
    // If there are no handlers (vector is empty), the packet is just dropped
    // TODO: factor out this clone-until-last-time pattern
    let mut iter = handlers.iter().peekable();
    loop {
      let handler = match iter.next() {
        None    => break,
        Some(h) => h
      };
      if iter.is_empty() {
        (&**handler).call((packet,));
        break;
      } else {
        (&**handler).call((packet.clone(),));
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
      Err(e) => debug!("packet could not be fowarded because {}", e),
    };
  }
}

/// Forwards a packet back into the network after rewriting its headers
/// Result status is whether packet was able to be forwarded
fn forward<A>(state: &super::State<A>, mut packet: packet::V) -> send::Result<()>
  where A: strategy::RoutingTable
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
  send::send_manual(row, packet).map_err(send::External)
}

/// Determine whether packet is destined for this node
fn is_packet_dst_local<A>(state: &super::State<A>, packet: &packet::V) -> bool
  where A: strategy::RoutingTable
{
  let dst = packet.borrow().get_destination();

  // TODO: factor out is_neighbor_addr and is_our_addr
  state.interfaces.iter()
    .any(|&super::InterfaceRow { local_ip, .. }| local_ip == dst)
}

struct IpDl<A>
  where A: strategy::RoutingTable + Send
{
  state: Arc<super::State<A>>,
}

impl<A> MyFn<(dl::Packet,), ()> for IpDl<A>
  where A: strategy::RoutingTable + Send
{
  fn call(&self, args: (dl::Packet,)) {
    let (packet,) = args;
    receive(&*self.state, packet);
  }
}

pub fn make_receive_callback<A>(state: Arc<super::State<A>>) -> dl::Handler
  where A: strategy::RoutingTable + Send
{
  box IpDl { state: state.clone() }
}
