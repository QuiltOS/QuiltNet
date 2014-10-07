use std::io::net::ip::IpAddr;

use network::ipv4::{IPState, RoutingRow};
//use network::ipv4::packet::{IpAddr, IPPacket};
use network::ipv4::packet::IPPacket;

//TODO: visibility?
pub fn send_data<'b>(state: &IPState, vip: IpAddr, protocol: u8, data: &'b [u8]){
    send(state, IPPacket::new(vip, protocol, data));
}

//TODO: visibility?
pub fn send<'b>(state: &IPState, packet: Box<IPPacket>) -> () {
    match state.routes.read().find(&packet.get_destination_address()) {
        None => (), // drop, no route to destination

        // Send packet to next hop towards destination
        // TODO: include loopback address in routing table
        Some(&RoutingRow { cost: cost, next_hop: next_hop }) => {
            match state.interfaces.find(&next_hop) {
                None => (), // drop, next hop isn't in our interface map

                // Tell interface to send packet bytes
                Some(&(_addr, interface)) => {
                    (*interface).send(*packet.to_bytes());
                }
            }
        }
    }
}
