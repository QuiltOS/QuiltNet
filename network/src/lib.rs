#![feature(unboxed_closures)]
#![feature(slice_patterns)]
#![feature(box_syntax)]
#![feature(question_mark)]

extern crate core;

#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate misc;

mod data_link {
  pub extern crate interface;
}

pub mod ipv4;
