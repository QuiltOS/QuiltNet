pub mod dummy;
pub mod ring;
pub mod send;
pub mod recv;

pub type BestPacketBuf = ring::RingPacketBuf;

pub trait PacketBuf
{
  fn new(init_seq_num: u32) -> Self;

  /// returns delta for the front
  fn add_vec  (&mut self, seq_num: u32, vec: Vec<u8>, start_off: uint) -> u32;

  /// returns delta for the front
  fn add_slice(&mut self, seq_num: u32, buf: &[u8]) -> u32;

  fn get_next_consume_seq(&self) -> u32;
  fn get_next_write_seq(&self) -> u32;
}
/*
pub trait PacketBufIter<'a>: PacketBuf
{
  type View:    Iterator<u8>;
  type Consume: Iterator<u8>;

  fn iter        (&'a     self) -> <Self as PacketBufIter<'a>>::View;
  fn consume_iter(&'a mut self) -> <Self as PacketBufIter<'a>>::Consume;
}
*/
