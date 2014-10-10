use std::sync::Arc;

use network::ipv4::{IPProtocolHandler, IPState};


/// Enables the given interface
pub fn up(ip_state: &IPState, interface: uint) -> Result<(), ()> {
    // no UFCS to make this concise
    match ip_state.get_interface(interface) {
        None                 => return Err(()),
        Some(&(_, _, ref i)) => (*i.write()).enable()
    };
    Ok(())
}

/// Disables the given interface
pub fn down(ip_state: &IPState, interface: uint) -> Result<(), ()> {
    match ip_state.get_interface(interface) {
        None                 => return Err(()),
        Some(&(_, _, ref i)) => (*i.write()).disable()
    };
    Ok(())
}

pub fn register_protocol_handler(ip_state: &IPState,
                                 proto_number: u8,
                                 handler: IPProtocolHandler)
{
    (*ip_state.protocol_handlers.write()).get_mut(proto_number as uint).push(handler);
}
