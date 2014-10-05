use network::ipv4::send;
use network::ipv4::IPState;
use network::ipv4::packet::IPPacket;


/// Called upon receipt of an IP packet:
/// If packet is destined for this node, deliver it to appropriate handlers
/// If packet is destined elsewhere, fix packet headers and forward
pub fn receive(state: &IPState, packet: IPPacket) {
    if is_packet_dst_local(packet) {
        // local handling
        match state.protocol_handlers.find(packet.header.protocol) {
            None => (), // drop, no handlers for protocol

            // Send packet to each handler for protocol
            Some(handlers) => {
                for handler in handlers {

                    // Handler also given IPState for
                    //  - inspection (CLI)
                    //  - modification (RIP)
                    handler(state, packet);
                }   
            }
        }
    } else {
        forward(state, packet);
    }
}

/// Forwards a packet back into the network after rewriting its headers
fn forward(state: &IPState, packet: IPPacket){

        // forward
        match fix_headers(packet) {
            None => (), // drop, TTL expired etc.

            // Send packet back into network
            Some(new_packet) => {
                send::send(state, new_packet);
            }
        }
}

/// Determine whether packet is destined for this node
fn is_packet_dst_local(state : IPState, packet : IPPacket) -> bool {
    return state.local_vips.contains_equiv(packet.header.destination_address);
}       

/// Return new packet with fixed headers
/// TODO: when to return None?
///     - When TTL expired
///     - When packet is invalid
fn fix_headers(packet : IPPacket) -> Option<IPPacket> {
    // decrement TTL
    // recompute checksum
    // TODO: etc
}
