use core::fmt::Debug;
use super::strategy;


/// Enables the given interface
pub fn up<'a, A, E>(ip_state: &super::State<'a, A, E>, interface: usize)
                   -> Result<(), ()>
  where A: strategy::RoutingTable<'a> + 'a,
        E: Debug + 'a
{
  // no UFCS to make this concise
  match ip_state.get_interface(interface) {
    None    => return Err(()),
    Some(x) => x.interface.write().unwrap().enable(),
  };
  Ok(())
}

/// Disables the given interface
pub fn down<'a, A, E>(ip_state: &super::State<'a, A, E>, interface: usize)
                     -> Result<(), ()>
  where A: strategy::RoutingTable<'a> + 'a,
        E: Debug + 'a
{
  match ip_state.get_interface(interface) {
    None    => return Err(()),
    Some(x) => x.interface.write().unwrap().disable(),
  };
  Ok(())
}

pub fn register_protocol_handler
  <'a, 'st: 'a, A, E>
  (ip_state: &'a super::State<'st, A, E>,
   proto_number: u8,
   handler: super::Handler<'st>)
  where A: strategy::RoutingTable<'st>
{
  ip_state.protocol_handlers.write().unwrap()[proto_number as usize].push(handler);
}
