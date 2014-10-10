use std::collections::HashMap;
use std::io::IoResult;
use std::sync::Arc;

use packet::ipv4::V as Ip;

use interface::Handler;

use data_link::{DLPacket, DLHandler};

use network::ipv4::send;
use network::ipv4::state::IPState;


/// Called upon receipt of an IP packet:
/// If packet is destined for this node, deliver it to appropriate handlers
/// If packet is destined elsewhere, fix packet headers and forward
pub fn receive(state: &IPState, packet: Ip) -> IoResult<()> {
    if is_packet_dst_local(state, &packet) {
        // local handling
        let handlers = &(*state.protocol_handlers.read())[packet.borrow().get_protocol() as uint];
        // If there are no handlers (vector is empty), the packet is just dropped
        // TODO: copy packet only if there are multiple handlers
        for handler in handlers.iter() {

            // Handler also given IPState for
            //  - inspection (CLI)
            //  - modification (RIP)
            (&**handler).call((packet.clone(),));
        }
    } else {
        try!(forward(state, packet));
    }
    Ok(())
}

/// Forwards a packet back into the network after rewriting its headers
/// Result status is whether packet was able to be forwarded
fn forward(state: &IPState, mut packet: Ip) -> IoResult<()> {
    // map Error because Fix_headers does not return IoError
    try!(fix_headers(&mut packet).map_err(|_| ::std::io::IoError {
        kind:   ::std::io::InvalidInput,
        desc:   "Packet had invalid headers",
        detail: None,
    }));
    try!(send::send(state, packet));
    Ok(())
}

/// Determine whether packet is destined for this node
fn is_packet_dst_local(state: &IPState, packet: &Ip) -> bool {
    state.interfaces.contains_key(&packet.borrow().get_destination())
}

/// Fix packet headers in place
///
/// Copy first if one wants to preserve the old packet.
/// Returns true if packet was valid / fixable.
fn fix_headers(packet: &mut Ip) -> Result<(), ()> {
    // decrement TTL
    // recompute checksum
    // TODO: etc
    // TODO ADD METHOD TO LIBRARY
    //try!(packet.verify_packet());

    decrement_ttl(packet);
    add_checksum(packet);
    Ok(())
}

/// Decrement packet's Time To Live field in place
fn decrement_ttl(_packet: &mut Ip) {
    // TTL_DEC
    //packet.set_time_to_live(packet.get_time_to_live() - 1);
}

/// Recompute checksum and add to header in place
/// TODO: actually compute IPv4 checksum
fn add_checksum(_packet: &mut Ip) {
    // TODO: STUB
    //packet.set_header_checksum(0);
}

pub fn make_receive_callback(state: Arc<IPState>) -> DLHandler {
    box |&: packet: DLPacket| -> () {
        match receive(&*state, Ip::new(packet)) {
            Ok(v)  => v,
            Err(e) => fail!(e),
        }
    }
}
