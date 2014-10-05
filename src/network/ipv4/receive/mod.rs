use std::collections::HashMap;
use std::sync::Arc;

use data_link::{DLPacket, DLHandler};

use network::ipv4::send;
use network::ipv4::IPState;
use network::ipv4::packet::IPPacket;


/// Called upon receipt of an IP packet:
/// If packet is destined for this node, deliver it to appropriate handlers
/// If packet is destined elsewhere, fix packet headers and forward
pub fn receive(state: &IPState, packet: Box<IPPacket>) {
    if is_packet_dst_local(state, &*packet) {
        // local handling
        let handlers = &state.protocol_handlers[packet.header.protocol as uint];
        // If there are no handlers (vector is empty), the packet is just dropped
        // TODO: copy packet only if there are multiple handlers
        for handler in handlers.iter() {

            // Handler also given IPState for
            //  - inspection (CLI)
            //  - modification (RIP)
            (*handler)(state, packet.clone_box());
        }
    } else {
        forward(state, packet);
    }
}

/// Forwards a packet back into the network after rewriting its headers
/// Result status is whether packet was able to be forwarded
fn forward(state: &IPState, packet: Box<IPPacket>) -> Result<(), ()> {
    try!(fix_headers(&*packet));
    send::send(state, packet);
}

/// Determine whether packet is destined for this node
fn is_packet_dst_local(state: &IPState, packet: &IPPacket) -> bool {
    state.interfaces.contains_key(&packet.header.destination_address)
}       

/// Fix packet headers in place
///
/// Copy first if one wants to preserve the old packet.
/// Returns true if packet was valid / fixable.
fn fix_headers(packet: &mut IPPacket) -> Result<(), ()> {
    // decrement TTL
    // recompute checksum
    // TODO: etc
}

// TODO: use Box<[u8]> instead of Vec<u8>
// TODO: real network card may consolidate multiple packets per interrupt
// TODO: lifetime for IPState probably needs fixing 
// TODO: Make some Sender type
pub type IPProtocolHandler = <'a> | &'a IPState, Box<IPPacket> |:Send -> ();


pub type ProtocolTable = Vec<Vec<IPProtocolHandler>>;


pub fn register_protocol(proto_table: &mut ProtocolTable, proto_number: u8, handler: IPProtocolHandler) {
    (*proto_table)[proto_number as uint].push(handler);
}

pub fn make_receive_callback(_state: Arc<IPState>) -> DLHandler {
    box |&: _packet: DLPacket| -> () {

    }
}
