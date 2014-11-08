use ipv4::{strategy, InterfaceRow, IpHandler, IpState};


/// Enables the given interface
pub fn up<A>(ip_state: &IpState<A>, interface: uint) -> Result<(), ()>
  where A: strategy::RoutingTable
{
  // no UFCS to make this concise
  match ip_state.get_interface(interface) {
    None                                      => return Err(()),
    Some(&InterfaceRow { ref interface, .. }) => (*interface.write()).enable()
  };
  Ok(())
}

/// Disables the given interface
pub fn down<A>(ip_state: &IpState<A>, interface: uint) -> Result<(), ()>
  where A: strategy::RoutingTable
{
  match ip_state.get_interface(interface) {
    None                                      => return Err(()),
    Some(&InterfaceRow { ref interface, .. }) => (*interface.write()).disable()
  };
  Ok(())
}

pub fn register_protocol_handler<A>(ip_state: &IpState<A>,
                                    proto_number: u8,
                                    handler: IpHandler)
  where A: strategy::RoutingTable
{
  (*ip_state.protocol_handlers.write())[proto_number as uint].push(handler);
}
