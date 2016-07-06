#![feature(unboxed_closures)]

extern crate core;

extern crate misc;

use core::convert::From;

use misc::interface as i;


// TODO: use Box<[u8]> instead of Vec<u8>
pub type Packet = Vec<u8>;
pub type Handler<'a> = i::Handler<'a, Packet>;

#[derive(Clone, Copy,
         PartialEq, Eq,
         PartialOrd, Ord,
         Hash, Debug)]
pub enum Error<E> {
  Disabled,
  External(E),
}

impl<E> From<E> for Error<E> {
  fn from(external: E) -> Error<E> {
    Error::External(external)
  }
}

pub type Result<T, E> = std::result::Result<T, self::Error<E>>;

pub trait Interface: i::Interface {
  /// Send packet with specified body
  fn send(&self, packet: Packet) -> self::Result<(), Self::Error>;

  /// Update the function called on an arriving packet
  fn update_recv_handler(&self, on_recv: Handler);

  //fn new(on_receive: |Vec<u8>| -> ()) -> Self;

  //fn stock(&mut Self, bufs: Box<[Vec<u8>]>);

  fn enable(&mut self);
  fn disable(&mut self);
  fn get_status(&self) -> bool;
}
