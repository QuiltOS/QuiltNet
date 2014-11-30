pub mod dummy;
pub mod ring;
pub mod vec_queue;

pub type BestPacketBuf = ring::PacketBuf;

pub trait PacketBuf
{
  fn new(init_seq_num: u32) -> Self;

  /// Returns the number of bytes from the payload that now exist in
  /// the buffer
  ///
  /// In particular, the bytes could have been added as a result of
  /// this call, or they could have already existed in the buffer
  #[inline]
  fn add_vec  (&mut self, seq_num: u32, vec: Vec<u8>, start_off: uint) -> u32;

  /// Return value as the same semantics as `add_vec`
  #[inline]
  fn add_slice(&mut self, seq_num: u32, buf: &[u8]) -> u32;

  fn get_next_consume_seq(&self) -> u32;
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


// TODO: make derived method in PacketBufIter

// TODO: cap at tail seq + 2^31 ?

/// seq number for the head of continuous data
pub fn get_next_write_seq(b: &BestPacketBuf) -> u32
{
  let tail = b.get_next_consume_seq();

  // number of continuous bytes
  // make sure not to consume
  let delta = b.iter().count();

  tail + delta as u32
}
