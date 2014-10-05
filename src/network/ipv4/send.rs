use network::ipv4::{IPState, RoutingRow};
use network::ipv4::packet::{IPAddr, IPPacket};

//TODO: visibility?
pub fn send(state: &IPState, vip: IPAddr, protocol: u8, data: [u8]){
    send_packet(state, IPPacket::new(vip, protocol, data));
}

//TODO: visibility?
pub fn send_packet(state: &IPState, packet: IPPacket) -> () {
    match state.routes.find(packet.header.destination_address) {
        None => (), // drop, no route to destination

        // Send packet to next hop towards destination
        // TODO: include loopback address in routing table
        Some(RoutingRow{ cost : cost, next_hop : next_hop }) => {
            match state.interfaces.find(next_hop) {
                None => (), // drop, next hop isn't in our interface map

                // Tell interface to send packet bytes
                Some(interface) => {
                    interface.send(packet.to_bytes());
                }
            }
        }
    }
}
