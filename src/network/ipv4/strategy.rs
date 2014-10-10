use std::io::net::ip::{Ipv4Addr, IpAddr};
use std::option::Option;

use packet::ipv4 as packet;

use super::{
    InterfaceRow,
};

pub trait RoutingTable: Send + Sync {

    fn init(&[InterfaceRow]) -> Self;

    fn lookup(&self, IpAddr) -> Option<IpAddr>;

    fn dump(&self);

}
