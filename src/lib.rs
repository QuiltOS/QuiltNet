#![feature(associated_types)]
#![feature(unboxed_closures)]
#![feature(unboxed_closure_sugar)]

#![cfg(test)]
#![feature(globs)]

extern crate core;
extern crate packet;

pub mod interface;

pub mod data_link;
pub mod network;
