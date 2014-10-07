#![feature(unboxed_closures)]

// This warning is really unimportant and annoying
#![allow(unused_imports)]

#![cfg(test)]
#![feature(globs)]

extern crate core;
extern crate packet;

pub mod interface;

pub mod data_link;
pub mod network;
