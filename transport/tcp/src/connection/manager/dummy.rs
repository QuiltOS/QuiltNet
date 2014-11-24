use std::slice::Items;
use std::iter::Map;

use super::{
  PacketBuf,
  //PacketBufIter,
};


pub struct DummyPacketBuf {
  dumb: [u8, ..2],
}


impl PacketBuf for DummyPacketBuf
{
  fn new(_init_seq_num: u32) -> DummyPacketBuf {
    DummyPacketBuf { dumb: [0, 0] }
  }

  fn add_vec(&mut self, _seq_num: u32, _vec: Vec<u8>, _start_off: uint) -> u32 { 0 }
  fn add_slice(&mut self, _seq_num: u32, _buf: &[u8]) -> u32 { 0 }

  fn get_next_seq(&self) -> u32 { 0 }
}


type ViewC<'a>    = Map<'a, &'a u8, u8, Items<'a, u8>>;
type ConsumeC<'a> = Map<'a, &'a u8, u8, Items<'a, u8>>;


//impl<'a> super::PacketBufIter<'a> for DummyPacketBuf
impl<'a> DummyPacketBuf
{
  type View    = ViewC<'a>;
  type Consume = ConsumeC<'a>;

  pub fn iter(&'a self) -> ViewC<'a> {
    self.dumb.iter().map(|x| *x)
  }

  pub fn consume_iter(&'a mut self) -> ConsumeC<'a> {
    self.dumb.iter().map(|x| *x)
  }
}
