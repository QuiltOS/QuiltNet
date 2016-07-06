use core::fmt::Debug;
use super::strategy;


/// Enables the given interface
pub fn up<A, E>(ip_state: &super::State<A, E>, interface: usize) -> Result<(), ()>
  where A: strategy::RoutingTable + 'static,
        E: Debug + 'static
{
  // no UFCS to make this concise
  match ip_state.get_interface(interface) {
    None    => return Err(()),
    Some(x) => x.interface.write().unwrap().enable(),
  };
  Ok(())
}

/// Disables the given interface
pub fn down<A, E>(ip_state: &super::State<A, E>, interface: usize) -> Result<(), ()>
  where A: strategy::RoutingTable + 'static,
        E: Debug + 'static
{
  match ip_state.get_interface(interface) {
    None    => return Err(()),
    Some(x) => x.interface.write().unwrap().disable(),
  };
  Ok(())
}

pub fn register_protocol_handler<A, E>(ip_state: &super::State<A, E>,
                                       proto_number: u8,
                                       handler: super::Handler)
  where A: strategy::RoutingTable
{
  ip_state.protocol_handlers.write().unwrap()[proto_number as usize].push(handler);
}
