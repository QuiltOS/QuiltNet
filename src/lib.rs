#![feature(unboxed_closures)]

#![cfg(test)]
#![feature(globs)]

extern crate core;
extern crate packet;

pub mod interface;

pub mod data_link;
pub mod network;
