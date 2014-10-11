use std::io::net::ip::{Ipv4Addr, IpAddr};
use std::option::Option;
use std::sync::Arc;

use packet::ipv4 as packet;

use super::{
    IpState,
    InterfaceRow,
};

pub trait RoutingTable: Send + Sync {

    fn init(&[InterfaceRow]) -> Self;

    fn lookup(&self, IpAddr) -> Option<IpAddr>;

    fn monitor(state: Arc<IpState<Self>>) -> ();

    fn dump(&self);

}
