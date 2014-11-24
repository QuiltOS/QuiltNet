use std::slice::Items;
use std::iter::Map;

pub use self::dummy::PacketBufIter;

pub mod dummy;
pub mod send;
pub mod recv;

pub trait PacketBuf
{
  fn new() -> Self;

  fn add_vec  (&mut self, seq_num: u32, vec: Vec<u8>, start_off: uint);
  fn add_slice(&mut self, seq_num: u32, buf: &[u8]);

  fn get_next_seq(&self) -> u32;
}
