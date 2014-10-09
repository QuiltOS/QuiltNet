use network::ipv4::state::IPState;

use super::receive::IPProtocolHandler;
/// Enables the given interface
pub fn up(ip_state: &mut IPState, interface: uint) -> Result<(), ()> {
    // no UFCS to make this concise
    match ip_state.get_interface_mut(interface) {
        None                     => return Err(()),
        Some(&(_, _, ref mut i)) => (*i).enable()
    };
    Ok(())
}

/// Disables the given interface
pub fn down(ip_state: &mut IPState, interface: uint) -> Result<(), ()> {
    match ip_state.get_interface_mut(interface) {
        None                     => return Err(()),
        Some(&(_, _, ref mut i)) => (*i).disable()
    };
    Ok(())
}

pub fn register_protocol_handler(ip_state: &mut IPState,
                                 proto_number: u8,
                                 handler: IPProtocolHandler)
{
    (ip_state.protocol_handlers).get_mut(proto_number as uint).push(handler);
}
