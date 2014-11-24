use std::slice::Items;
use std::iter::Map;

use ring_buf::{mod, RingBuf};

use super::{
  PacketBuf,
  //PacketBufIter,
};

//type ViewC<'a>    = Map<'a, &'a u8, u8, Items<'a, u8>>;
//type ConsumeC<'a> = Map<'a, &'a u8, u8, Items<'a, u8>>;


pub struct RingPacketBuf {
  seq:  u32,
  ring: RingBuf,
}

impl PacketBuf for RingPacketBuf
{
  fn new(init_seq_num: u32) -> RingPacketBuf {
    RingPacketBuf {
      seq:  init_seq_num,
      ring: RingBuf::new(0b_1_00_0000_0000_0000), //2 ^ 14
    }
  }

  fn add_vec(&mut self, seq_num: u32, vec: Vec<u8>, start_off: uint) {
    self.add_slice(seq_num, vec.as_slice()[start_off..])
  }

  fn add_slice(&mut self, seq_num: u32, buf: &[u8]) {
    let delta: u64 = (seq_num as u64) - (self.seq as u64);

    let n = self.ring.write(buf);
    assert_eq!(n, buf.len());
  }

  fn get_next_seq(&self) -> u32 { self.seq }
}
