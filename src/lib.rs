#![feature(unboxed_closures)]
#![feature(slicing_syntax)]
#![cfg(test)]
// This warning is really unimportant and annoying
#![allow(unused_imports)]
#![feature(globs)]

extern crate core;
extern crate packet;

pub mod interface;
pub mod drivers;
pub mod data_link;
pub mod network;
pub mod utils;
