use network::ipv4::IPState;
use network::ipv4::headers::IPPacket;

pub struct ProtocolHandlers;

// TODO: use Box<[u8]> instead of Vec<u8>
// TODO: real network card may consolidate multiple packets per interrupt.
// TODO: Make some Sender type
pub type IPHandler = u32; //Box<Fn<(&IPState, Box<IPPacket>), ()> + Send + 'static>;
    
//fn register_protocol(u8, 
