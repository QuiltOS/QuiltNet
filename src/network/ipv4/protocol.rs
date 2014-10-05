use network::ipv4::IPState;
use network::ipv4::packet::IPPacket;

pub struct ProtocolHandlers;


// TODO: use Box<[u8]> instead of Vec<u8>
// TODO: real network card may consolidate multiple packets per interrupt.
// TODO: Make some Sender type
pub type IPProtocolHandler = Fn<IPState, IPPacket> + 'static;
   

//TODO: lock handlers?
pub fn register_protocol_handler(state: IPState, protocol: u8, handler: IPProtocolHandler){
    state.protocol_handlers.insert_or_update_with(state.protocol_handlers, protocol, vec!(handler),
                                                  |p, handlers| handlers.push(handler));
}

