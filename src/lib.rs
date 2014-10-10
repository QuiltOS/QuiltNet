// This warning is really unimportant and annoying
#![allow(unused_imports)]
#![allow(unknown_features)]

#![feature(unboxed_closures)]
// for Anson's rustc
#![feature(slicing_syntax)]
// for tests
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
