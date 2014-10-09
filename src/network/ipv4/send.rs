extern crate packet;

use std::io::{IoError, IoResult, IoUnavailable};
use std::io::net::ip::IpAddr;

use self::packet::ipv4::V as Ip;

use network::ipv4::state::{IPState, RoutingRow};
//use network::ipv4::packet::{IpAddr, IPPacket};

//TODO: visibility?
pub fn send_data(state: &IPState, vip: IpAddr, protocol: u8, data: &[u8]) -> IoResult<()> {
    //TODO: make from for header in newly allocated vec, set fields
    println!("send:: sending {} {} {}", vip, protocol, data);
    let p = Ip::from_body(vip, protocol, data);
    println!("build packet {}", p);
    try!(send(state, p));
    Ok(())
}

//TODO: visibility?
pub fn send(state: &IPState, packet: Ip) -> IoResult<()> {
    match state.routes.read().find(&packet.borrow().dest()) {
        None => (), // drop, no route to destination

        // Send packet to next hop towards destination
        // TODO: include loopback address in routing table
        // TODO: include broadcast interface w/ overloaded send fn
        Some(&RoutingRow { cost: _cost, next_hop: next_hop, learned_from: _learned_from}) => {
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

/// Broadcast data to all known nodes
pub fn neighborcast(state: &IPState, protocol: u8, data: Vec<u8>) -> IoResult<()> {
    for dst in state.interfaces.keys() {
        let err = send_data(state, *dst, protocol, data.as_slice());
        match err {
            Err(IoError { kind: IoUnavailable, .. }) => continue,  // ignore down interface
            _                                        => try!(err), // otherwise handle errors as usual
        };
    }
    Ok(())
}
