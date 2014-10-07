use std::io::net::ip::IpAddr;

use packet::parser::Ip;

use network::ipv4::{IPState, RoutingRow};
//use network::ipv4::packet::{IpAddr, IPPacket};

//TODO: visibility?
pub fn send_data<'b>(state: &IPState, vip: IpAddr, protocol: u8, data: &'b [u8]) {
    let p = Ip::new(data.to_vec());
    //TODO: set fields
}

//TODO: visibility?
pub fn send<'b>(state: &IPState, packet: Ip) -> () {
    match state.routes.read().find(&packet.dest()) {
        None => (), // drop, no route to destination

        // Send packet to next hop towards destination
        // TODO: include loopback address in routing table
        Some(&RoutingRow { cost: cost, next_hop: next_hop }) => {
            match state.interfaces.find(&next_hop) {
                None => (), // drop, next hop isn't in our interface map

                // Tell interface to send packet bytes
                Some(index) => {
                    let (_, _, ref interface) = state.interface_vec[*index];
                    interface.send(packet.as_vec());
                }
            }
        }
    }
}
