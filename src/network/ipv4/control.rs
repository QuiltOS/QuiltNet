use std::sync::Arc;

use network::ipv4::{strategy, IpHandler, IpState};


/// Enables the given interface
pub fn up<A>(ip_state: &IpState<A>, interface: uint) -> Result<(), ()>
    where A: strategy::RoutingTable
{
    // no UFCS to make this concise
    match ip_state.get_interface(interface) {
        None                 => return Err(()),
        Some(&(_, _, ref i)) => (*i.write()).enable()
    };
    Ok(())
}

/// Disables the given interface
pub fn down<A>(ip_state: &IpState<A>, interface: uint) -> Result<(), ()>
    where A: strategy::RoutingTable
{
    match ip_state.get_interface(interface) {
        None                 => return Err(()),
        Some(&(_, _, ref i)) => (*i.write()).disable()
    };
    Ok(())
}

pub fn register_protocol_handler<A>(ip_state: &IpState<A>,
                                    proto_number: u8,
                                    handler: IpHandler)
    where A: strategy::RoutingTable
{
    (*ip_state.protocol_handlers.write()).get_mut(proto_number as uint).push(handler);
}
