//#![feature(unboxed_closures)]
#![feature(slicing_syntax)]

// for tests
//#![feature(globs)]

#![feature(phase)]

#[cfg(not(ndebug))]
#[phase(plugin, link)]
extern crate log;

#[phase(plugin, link)]
extern crate misc;

mod data_link {
  extern crate interface;
}

pub mod ipv4;
