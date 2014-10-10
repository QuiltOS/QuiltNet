use std::io::{IoError, IoResult, IoUnavailable, NotConnected};
use std::io::net::ip::{Ipv4Addr, IpAddr};

use packet::ipv4 as packet;

use network::ipv4::{strategy, IpState, InterfaceRow};
//use network::ipv4::packet::{IpAddr, IpPacket};

//TODO: visibility?
pub fn send_data<A>(state: &IpState<A>, vip: IpAddr, protocol: u8, data: &[u8]) -> IoResult<()>
    where A: strategy::RoutingTable
{

    println!("send:: sending {} {} {}", vip, protocol, data);
    let p = packet::V::from_body(vip, protocol, data);
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
pub fn send<A>(state: &IpState<A>, mut packet: packet::V) -> IoResult<()>
    where A: strategy::RoutingTable
{
    match packet.borrow().get_destination() {
        // broadcast,
        Ipv4Addr(0,0,0,0) =>
            for row in state.interfaces.iter() {
                try!(send_per_interface(packet.clone(), row));
            },
        // neighbor cast
        Ipv4Addr(0,0,0,1) =>
            for row in state.interfaces.iter() {
                let &(_, dest, _) = row;
                let _ = packet.borrow_mut().set_destination(dest);
                try!(send_per_interface(packet.clone(), row));
            },
        _ => match state.routes.lookup(packet.borrow().get_destination()) {
            None => (), // drop, no route to destination

            // Send packet to next hop towards destination
            // TODO: include loopback address in routing table
            // TODO: include broadcast interface w/ overloaded send fn
            Some(next_hop) => {
                println!("Found route through {}", next_hop);
                match state.ip_to_interface.find(&next_hop) {
                    // drop, next hop isn't in our interface map
                    None => return Err(NO_ROUTE_ERROR.clone()),
                    // Tell interface to send packet bytes
                    Some(index) => {
                        try!(send_per_interface(packet, &state.interfaces[*index]));
                    }
                }
            }
        }
    }
    Ok(())
}

fn send_per_interface(mut packet: packet::V, row: &InterfaceRow) -> IoResult<()> {
    let &(src, _, ref interface) = row;
    let _ = packet.borrow_mut().set_source(src); // ip for this interface
    interface.write().send(packet.as_vec())
}


/// Broadcast data to all known nodes
pub fn neighborcast<A>(state: &IpState<A>, protocol: u8, data: Vec<u8>) -> IoResult<()>
    where A: strategy::RoutingTable
{
    for dst in state.ip_to_interface.keys() {
        let err = send_data(state, *dst, protocol, data.as_slice());
        match err {
            // ignore down interface
            Err(IoError { kind: IoUnavailable, .. }) => continue,
            // otherwise handle errors as usual
            _                                        => try!(err),
        };
    }
    Ok(())
}
