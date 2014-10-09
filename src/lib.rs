#![feature(unboxed_closures)]

#![allow(unknown_features)]
#![feature(slicing_syntax)]

// This warning is really unimportant and annoying
#![allow(unused_imports)]

#![cfg(test)]
#![feature(globs)]

extern crate core;
extern crate rustrt;
extern crate native;

extern crate packet;

pub mod interface;
pub mod drivers;
pub mod data_link;
pub mod network;
pub mod utils;
