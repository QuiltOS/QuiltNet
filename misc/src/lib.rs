#![feature(macro_rules)]
#![feature(phase)]

#[cfg(not(ndebug))]
#[phase(plugin, link)]
extern crate log;


#[cfg(ndebug)]
#[macro_escape]
pub mod mock_debug;

pub mod interface;
pub mod state_machine;
