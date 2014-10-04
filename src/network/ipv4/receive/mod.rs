use interface::Handler;

use network::ipv4::IPState;
use network::ipv4::headers::IPPacket;
use network::ipv4::protocol::IPHandler;

pub mod forward;

pub struct ProtocolTable(u8, );

//fn make_receive_callback(state: &IPState) -> Handler {
//    box |&: _packet: Vec<u8>| -> () {
//        
//    }
//}
