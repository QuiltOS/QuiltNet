extern crate packet;

use std::io::{IoError, IoResult, IoUnavailable, NotConnected};
use std::io::net::ip::{Ipv4Addr, IpAddr};

use self::packet::ipv4::V as Ip;
use self::packet::ipv4::A as IpSlice;

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

static NO_ROUTE_ERROR: IoError = IoError {
    kind: NotConnected,
    desc: "No routing table entry for this packet",
    detail: None,
};

//TODO: visibility?
//TODO: move, not copy, packet for final interface
pub fn send(state: &IPState, mut packet: Ip) -> IoResult<()> {
    match packet.borrow().get_destination() {
        // broadcast,
        Ipv4Addr(0,0,0,0) =>
            for &(_, _, ref interface) in state.interface_vec.iter() {
                try!(interface.write().send(packet.clone().as_vec()));
            },
        Ipv4Addr(0,0,0,1) =>
            for &(_, dest, ref interface) in state.interface_vec.iter() {
                packet.borrow_mut().set_destination(dest);
                try!(interface.write().send(packet.clone().as_vec()));
            },
        dest => match state.routes.read().find(&packet.borrow().get_destination()) {
            None => (), // drop, no route to destination

            // Send packet to next hop towards destination
            // TODO: include loopback address in routing table
            // TODO: include broadcast interface w/ overloaded send fn
            Some(&RoutingRow { next_hop: next_hop, .. }) => {
                match state.interfaces.find(&next_hop) {
                    // drop, next hop isn't in our interface map
                    None => (), 
                    // Tell interface to send packet bytes
                    Some(index) => {
                        let (_, _, ref interface) = state.interface_vec[*index];
                        try!(interface.write().send(packet.as_vec()));
                    }
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
