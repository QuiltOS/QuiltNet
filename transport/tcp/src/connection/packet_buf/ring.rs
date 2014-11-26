use std::slice::Items;
use std::cmp;
use std::iter::{Map, Scan};

use ring_buf::{mod, RingBuf};


#[deriving(Show)]
pub struct PacketBuf {
  tail_seq: u32,
  ring:    RingBuf,
}


impl super::PacketBuf for PacketBuf
{
  fn new(init_seq_num: u32) -> PacketBuf {
    PacketBuf {
      tail_seq: init_seq_num,
      ring:     RingBuf::new(1 << 14), // used to be 2^14
    }
  }

  fn get_next_consume_seq(&self) -> u32 { self.tail_seq }

  fn add_vec(&mut self, seq_num: u32, vec: Vec<u8>, start_off: uint) -> u32 {
    self.add_slice(seq_num, vec.as_slice()[start_off..])
  }

  fn add_slice(&mut self, seq_num: u32, buf: &[u8]) -> u32
  {
    let delta: i64 =
      (seq_num as i64) -
      ((self.tail_seq + self.ring.readable_len() as u32) as i64);
    debug!("delta: {}", delta);

    if delta == 0
    {
      let bytes_fit = cmp::min(self.ring.writable_len(), buf.len());

      self.ring.write(buf[..bytes_fit]) as u32
    }
    else if // overlaps, but is not completely contained
      (delta < 0) &&
      (buf.len() - (-delta) as uint) < self.ring.writable_len() &&
      ((-delta) as uint) < buf.len()
    {
      debug!("overlaps");
      self.ring.write(buf[-delta as uint..]) as u32
    }
    else // out of order or redundant/contained
    { debug!("not writing anything"); 0 }
  }
}


type ViewC<'a>    = ring_buf::View<'a>;
type ConsumeC<'a> = Scan<'a, u8, u8, ring_buf::Consume<'a>, &'a mut u32>;


//impl<'a> PacketBufIter<'a> for PacketBuf
impl<'a>  PacketBuf
{
  //type View    = ViewC<'a>;
  //type Consume = ConsumeC<'a>;

  #[inline]
  pub fn iter(&'a self) -> ViewC<'a> {
    self.ring.iter()
  }

  #[inline]
  pub fn consume_iter(&'a mut self) -> ConsumeC<'a> {

    // TODO close over len instead
    let inc: |&mut &mut u32, u8|:'a -> Option<u8> = |ptr, b| {
      **ptr = **ptr + 1; // wrap around is inplicit
      Some(b)
    };

    self.ring.consume_iter()
      .scan(&mut self.tail_seq, inc)
  }
}
