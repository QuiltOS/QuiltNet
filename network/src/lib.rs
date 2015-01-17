#![allow(unstable)]

#![feature(unboxed_closures)]
#![feature(box_syntax)]

#[macro_use] #[no_link]
extern crate log_ng;

//#[cfg(any(log_level = "error",
//          log_level = "warn",
//          log_level = "info",
//          log_level = "debug",
//          log_level = "trace"))]
extern crate log_ng;

#[macro_use]
extern crate misc;

mod data_link {
  extern crate interface;
}

pub mod ipv4;
