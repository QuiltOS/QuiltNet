use std::io::IoResult;
use std::io::net::ip::IpAddr;

use packet::parser::Ip;

use network::ipv4::{IPState, RoutingRow};
//use network::ipv4::packet::{IpAddr, IPPacket};

//TODO: visibility?
pub fn send_data(_state: &IPState, _vip: IpAddr, _protocol: u8, data: &[u8]) {
    //TODO: make from for header in newly allocated vec, set fields
    let _p = Ip::new(data.to_vec());
}

//TODO: visibility?
pub fn send<'b>(state: &IPState, packet: Ip) -> IoResult<()> {
    match state.routes.read().find(&packet.dest()) {
        None => (), // drop, no route to destination

        // Send packet to next hop towards destination
        // TODO: include loopback address in routing table
        Some(&RoutingRow { cost: _cost, next_hop: next_hop }) => {
            match state.interfaces.find(&next_hop) {
                None => (), // drop, next hop isn't in our interface map

                // Tell interface to send packet bytes
                Some(index) => {
                    let (_, _, ref interface) = state.interface_vec[*index];
                    try!(interface.send(packet.as_vec()));
                }
            }
        }
    }
    Ok(())
}
