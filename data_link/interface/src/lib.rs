#![allow(unstable)]

#![feature(unboxed_closures)]

extern crate misc;

use std::io::IoError;

pub use misc::interface as i;


// TODO: use Box<[u8]> instead of Vec<u8>
pub type Packet = Vec<u8>;
pub type Handler = i::Handler<Packet>;

// TODO: use associated type instead of IoError
#[derive(PartialEq, Eq, Clone, Show)]
pub enum Error {
  Disabled,
  External(IoError),
}

pub type Result<T> = std::result::Result<T, self::Error>;

pub trait Interface: i::Interface {

  /// Send packet with specified body
  fn send(&self, packet: Packet) -> self::Result<()>;

  /// Update the function called on an arriving packet
  fn update_recv_handler(&self, on_recv: Handler);

  //type Err;

  //fn new(on_receive: |Vec<u8>| -> ()) -> Self;

  //fn stock(&mut Self, bufs: Box<[Vec<u8>]>);

  fn enable(&mut self);
  fn disable(&mut self);
  fn get_status(&self) -> bool;
}
