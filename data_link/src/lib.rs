// This warning is really unimportant and annoying
#![allow(unused_imports)]
#![allow(unknown_features)]

//#![feature(unboxed_closures)]
// for Anson's rustc
#![feature(slicing_syntax)]
// for tests
#![feature(globs)]

extern crate misc;

use std::io::IoResult;

pub use misc::interface as i;

pub mod udp_mock;

// TODO: use Box<[u8]> instead of Vec<u8>
pub type Packet = Vec<u8>;

pub type Handler = i::Handler<Packet>;

pub trait Interface: i::Interface {

  /// Send packet with specified body
  fn send(&self, packet: Packet) -> IoResult<()>;

  /// Update the function called on an arriving packet
  fn update_recv_handler(&self, on_recv: Handler);

  //type Err;

  //fn new(on_receive: |Vec<u8>| -> ()) -> Self;

  //fn stock(&mut Self, bufs: Box<[Vec<u8>]>);

  //fn kill(&Self);

  fn enable(&mut self);
  fn disable(&mut self);
  fn get_status(&self) -> bool;
}
