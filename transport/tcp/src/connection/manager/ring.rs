use std::slice::Items;
use std::iter::{Map, Scan};

use ring_buf::{mod, RingBuf};

use super::{
  PacketBuf,
  //PacketBufIter,
};


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

  fn add_vec(&mut self, seq_num: u32, vec: Vec<u8>, start_off: uint) -> u32 {
    self.add_slice(seq_num, vec.as_slice()[start_off..])
  }

  fn add_slice(&mut self, seq_num: u32, buf: &[u8]) -> u32
  {
    let delta: u64 = (seq_num as u64) - (self.seq as u64);

    (if     // tacks on perfectly
      (delta == 0) &&
      (delta as uint + buf.len()) < self.ring.window_size()
    {
      self.ring.write(buf)
    }
    else if // overlaps, but is not completely contained
      (delta < 0) &&
      (buf.len() - (-delta) as uint) < self.ring.window_size() &&
      ((-delta) as uint) < buf.len()
    {
      self.ring.write(buf[-delta as uint..])
    }
    else // out of order or redundant/contained
    { 0 }
    as u32)
  }

  fn get_next_seq(&self) -> u32 { self.seq }
}


type ViewC<'a>    = Map <'a, &'a u8, u8, ring_buf::View<'a>>;
type ConsumeC<'a> = Scan<'a,     u8, u8, ring_buf::Consume<'a>, &'a mut u32>;


//impl<'a> PacketBufIter<'a> for RingPacketBuf
impl<'a>  RingPacketBuf
{
  type View    = ViewC<'a>;
  type Consume = ConsumeC<'a>;

  pub fn iter(&'a self) -> ViewC<'a> {
    self.ring.iter().map(|x| *x)
  }

  pub fn consume_iter(&'a mut self) -> ConsumeC<'a> {

    // TODO close over len instead
    let inc: |&mut &mut u32, u8|:'a -> Option<u8> = |ptr, b| {
      **ptr = **ptr + 1; // wrap around is inplicit
      Some(b)
    };

    self.ring.consume_iter()
      .scan(&mut self.seq, inc)
  }
}
