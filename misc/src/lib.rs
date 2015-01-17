#![allow(unstable)]

#![feature(unboxed_closures)]

#[macro_use] #[no_link]
extern crate log_ng;

//#[cfg(any(log_level = "error",
//          log_level = "warn",
//          log_level = "info",
//          log_level = "debug",
//          log_level = "trace"))]
extern crate log_ng;


pub mod interface;
pub mod state_machine;
