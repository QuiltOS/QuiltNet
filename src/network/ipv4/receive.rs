use std::collections::HashMap;
use std::io::IoResult;
use std::sync::Arc;

use packet::ipv4 as packet;

use interface::{MyFn, Handler};

use data_link::{DLPacket, DLHandler};

use network::ipv4::send;
use network::ipv4::IpState;


/// Called upon receipt of an IP packet:
/// If packet is destined for this node, deliver it to appropriate handlers
/// If packet is destined elsewhere, fix packet headers and forward
fn receive(state: &IpState, buf: Vec<u8>) -> IoResult<()> {

    let packet = match packet::validate(buf.as_slice()) {
        Ok(_)  => packet::V::new(buf),
        Err(e) => {
            println!("dropping incomming packet because {}", e);
            return Ok(())
        },
    };

    println!("checking if packet is local");
    if is_packet_dst_local(state, &packet) {
        println!("Packet is local! {}", packet);
        // local handling
        let handlers = &(*state.protocol_handlers.read())
            [packet.borrow().get_protocol() as uint];
        // If there are no handlers (vector is empty), the packet is just dropped
        // TODO: copy packet only if there are multiple handlers
        for handler in handlers.iter() {
            println!("Handing to handler");
            // Handler also given IpState for
            //  - inspection (CLI)
            //  - modification (Rip)
            (&**handler).call((packet.clone(),));
        }
    } else {
        println!("packet is not local!");
        try!(forward(state, packet));
    }
    Ok(())
}

/// Forwards a packet back into the network after rewriting its headers
/// Result status is whether packet was able to be forwarded
fn forward(state: &IpState, mut packet: packet::V) -> IoResult<()> {
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
fn is_packet_dst_local(state: &IpState, packet: &packet::V) -> bool {
    let dst = &packet.borrow().get_destination();
    println!("after borrow: {}", dst);
    state.ip_to_interface.contains_key(dst)
}

/// Fix packet headers in place
///
/// Copy first if one wants to preserve the old packet.
/// Returns true if packet was valid / fixable.
fn fix_headers(packet: &mut packet::V) -> Result<(), ()> {
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
fn decrement_ttl(_packet: &mut packet::V) {
    // TTL_DEC
    //packet.set_time_to_live(packet.get_time_to_live() - 1);
}

/// Recompute checksum and add to header in place
/// TODO: actually compute Ipv4 checksum
fn add_checksum(_packet: &mut packet::V) {
    // TODO: STUB
    //packet.set_header_checksum(0);
}

struct IpDl {
    state: Arc<IpState>,
}

impl MyFn<(DLPacket,), ()> for IpDl {
    fn call(&self, args: (DLPacket,)) {
        let (packet,) = args;
        println!("in callback");
        match receive(&*self.state, packet) {
            Ok(v)  => v,
            Err(e) => fail!(e),
        }
    }
}

pub fn make_receive_callback(state: Arc<IpState>) -> DLHandler {
    box IpDl { state: state.clone() }
}
