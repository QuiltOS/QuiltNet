use std::slice::Items;
use std::iter::Map;

use super::{
  PacketBuf,
  //PacketBufIter,
};

type ViewC<'a>    = Map<'a, &'a u8, u8, Items<'a, u8>>;
type ConsumeC<'a> = Map<'a, &'a u8, u8, Items<'a, u8>>;


pub struct DummyPacketBuf {
  dumb: [u8, ..2],
}

impl PacketBuf for DummyPacketBuf
{
  fn new(_init_seq_num: u32) -> DummyPacketBuf {
    DummyPacketBuf { dumb: [0, 0] }
  }

  fn add_vec(&mut self, _seq_num: u32, _vec: Vec<u8>, _start_off: uint) {}
  fn add_slice(&mut self, _seq_num: u32, _buf: &[u8]) {}

  fn get_next_seq(&self) -> u32 { 0 }
}


pub trait PacketBufIter<'a>: PacketBuf
{
  type View:    Iterator<u8>;
  type Consume: Iterator<u8>;

  fn iter        (&'a     self) -> <Self as PacketBufIter<'a>>::View;
  fn consume_iter(&'a mut self) -> <Self as PacketBufIter<'a>>::Consume;
}


impl<'a> super::PacketBufIter<'a> for DummyPacketBuf
{
  type View    = ViewC<'a>;
  type Consume = ConsumeC<'a>;

  fn iter(&'a self) -> ViewC<'a> {
    self.dumb.iter().map(|x| *x)
  }

  fn consume_iter(&'a mut self) -> ConsumeC<'a> {
    self.dumb.iter().map(|x| *x)
  }
}
