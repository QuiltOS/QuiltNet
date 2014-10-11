use std::io::net::ip::IpAddr;
use std::sync::Arc;

use network::ipv4::{strategy, control, IpState, IpHandler};

use packet::ipv4::V as Ip;

use interface::MyFn;

use super::{
    RipTable,
    RipRow,
};

struct RipHandler { _state: Arc<IpState<RipTable>> }

impl MyFn<(Ip,), ()> for RipHandler {

    fn call(&self, _args: (Ip,)) {
    }
}


/// Runs simple debug handler, printing out all packets received for the given protocols
pub fn register(state: Arc<IpState<RipTable>>) {
    control::register_protocol_handler(
        &*state,
        200,
        box RipHandler { _state: state.clone() })
}

/// send to everybody but the receiver of the poison
pub fn send(_state: &IpState<RipTable>, _poison: IpAddr, _row: &RipRow) {
    
}
