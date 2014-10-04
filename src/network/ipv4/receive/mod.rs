use std::collections::HashMap;
use std::sync::Arc;

use interface::Handler;

use network::ipv4::IPState;
use network::ipv4::headers::IPPacket;

pub mod forward;



// TODO: use Box<[u8]> instead of Vec<u8>
// TODO: real network card may consolidate multiple packets per interrupt.
// TODO: Make some Sender type
pub type IPHandler = Box<Fn<(Arc<IPState>, Box<IPPacket>), ()> + Send + 'static>;

pub type ProtocolTable = HashMap<u8, IPHandler>;


pub fn register_protocol(proto_table: &mut ProtocolTable, proto_number: u8, handler: IPHandler) {
    proto_table.insert(proto_number, handler);
}

pub fn make_receive_callback(_state: Arc<IPState>) -> Handler {
    box |&: _packet: IPPacket| -> () {

    }
}
