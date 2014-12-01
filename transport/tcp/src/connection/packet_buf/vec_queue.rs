extern crate cyclic_order;

use std::cmp;
use std::collections::dlist;
use std::collections::DList;
use std::slice;
use std::iter::{
  mod,
  Chain,
  Counter,
  Filter,
  FilterMap,
  FlatMap,
  Map,
  Rev,
  Scan,
  Skip,
  SkipWhile,
  Peekable,
  TakeWhile,
  Zip,
};
use std::num::Int;

use self::cyclic_order::{CyclicOrd, PartialCyclicOrd};
use self::cyclic_order::CyclicOrdering::*;
use self::cyclic_order::linked_list::{mod, CutQueue};

#[deriving(PartialEq, Eq, Show)]
struct Tagged {
  seq:    u32,
  offset: uint,
  buf:    Vec<u8>,
}

impl PartialCyclicOrd for Tagged {
  fn is_clockwise(&self, them: &Tagged, other: &Tagged) -> bool {
    self.cyclic_cmp(them, other) == Clockwise
  }
}


impl CyclicOrd for Tagged
{
  fn cyclic_cmp(&self, them: &Tagged, other: &Tagged) -> cyclic_order::CyclicOrdering {
    match self.tail().cyclic_cmp(&them.tail(), &other.tail()) {
      Degenerate => self.head().cyclic_cmp(&them.head(), &other.head()),
      otherwise  => otherwise
    }
  }
}


impl Tagged
{
  #[inline]
  fn new(seq_num: u32, vec: Vec<u8>, start_off: uint) -> Tagged
  {
    let u32_max: u32 = Int::max_value();

    let node = Tagged {
      seq:    seq_num,
      offset: start_off,
      buf:    vec,
    };

    assert!(start_off < node.len());
    assert!((node.len() - start_off) < u32_max as uint);

    node
  }

  #[inline]
  fn len(&self) -> uint
  {
    self.buf.len() - self.offset
  }

  #[inline]
  fn head(&self) -> u32
  {
    self.seq + (self.len() - self.offset) as u32
  }

  #[inline]
  fn tail(&self) -> u32
  {
    self.seq
  }
}

impl ::std::slice::AsSlice<u8> for Tagged
{
  #[inline]
  fn as_slice<'a>(&'a self) -> &'a [u8]
  {
    self.buf.as_slice()[self.offset..]
  }
}

mod tagged_test {
  /*
  use super::cylic_order::{partial_axioms, total_axioms};

  #[quickcheck]
  fn partial_cyclicity(a: Tagged, b: Tagged, c: Tagged) -> bool {
    partial_axioms::cyclicity(&a, &b, &c)
  }

  #[quickcheck]
  fn partial_antisymmetry(a: Tagged, b: Tagged, c: Tagged) -> bool {
    partial_axioms::antisymmetry(&a, &b, &c)
  }

  #[quickcheck]
  fn partial_transitivity(a: Tagged, b: Tagged, c: Tagged, d: Tagged) -> bool {
    partial_axioms::transitivity(&a, &b, &c, &d)
  }


  #[quickcheck]
  fn total_cyclicity(a: Tagged, b: Tagged, c: Tagged) -> bool {
    total_axioms::cyclicity(&a, &b, &c)
  }

  #[quickcheck]
  fn total_antisymmetry(a: Tagged, b: Tagged, c: Tagged) -> bool {
    total_axioms::antisymmetry(&a, &b, &c)
  }

  #[quickcheck]
  fn total_transitivity(a: Tagged, b: Tagged, c: Tagged, d: Tagged) -> bool {
    total_axioms::transitivity(&a, &b, &c, &d)
  }

  #[quickcheck]
  fn total_totality(a: Tagged, b: Tagged, c: Tagged) -> bool {
    total_axioms::totality(&a, &b, &c)
  }

  #[quickcheck]
  fn super_trait_cohesion(a: Tagged, b: Tagged, c: Tagged) -> bool {
    total_axioms::super_trait_cohesion(&a, &b, &c)
  }
  */
}




#[deriving(Show)]
pub struct PacketBuf {
  tail_seq: u32,
  data:     DList<Tagged>,
}


impl super::PacketBuf for PacketBuf
{
  fn new(init_seq_num: u32) -> PacketBuf {
    PacketBuf {
      tail_seq: init_seq_num,
      data:     DList::new(),
    }
  }

  fn get_next_consume_seq(&self) -> u32 { self.tail_seq }

  fn add_slice(&mut self, seq_num: u32, buf: &[u8]) -> u32
  {
    self.add_vec(seq_num, buf.to_vec(), 0)
  }

  fn add_vec(&mut self, seq_num: u32, vec: Vec<u8>, start_off: uint) -> u32
  {
    let node = Tagged::new(seq_num, vec, start_off);
    let ret = node.head() - node.tail(); // always accept everything!
    self.data.insert_cyclic(node);
    ret
  }
}


// TODO: reuse code between the two iterators

pub struct ViewIter<'a> {
  expected_seq: u32,
  tagged_iter:  Peekable<&'a Tagged, dlist::Items<'a, Tagged>>,
}

impl<'a> Iterator<u8> for ViewIter<'a>
{
  #[inline]
  fn next(&mut self) -> Option<u8>
  {
    loop {
      {
        let next = match self.tagged_iter.peek() {
          None    => return None,
          Some(s) => s,
        };

        if
          self.expected_seq == next.tail() ||
          next.tail().is_clockwise(&self.expected_seq, &next.head())
        {
          let tagged = next;

          let ret = Some(tagged.as_slice()[(self.expected_seq - tagged.tail()) as uint]);
          self.expected_seq += 1;

          return ret;
        }
      };

      let _ = self.tagged_iter.next();
    }
  }
}

pub struct ConsumeIter<'a>(&'a mut PacketBuf);

impl<'a> Iterator<u8> for ConsumeIter<'a>
{
  #[inline]
  fn next(&mut self) -> Option<u8>
  {
    loop {
      {
        let next = match self.0.data.front() {
          None    => return None,
          Some(s) => s,
        };

        if
          self.0.tail_seq == next.tail() ||
          next.tail().is_clockwise(&self.0.tail_seq, &next.head())
        {
          let tagged = next;

          let ret = Some(tagged.as_slice()[(self.0.tail_seq - tagged.tail()) as uint]);
          self.0.tail_seq += 1;

          return ret;
        }
      };

      let _ = self.0.data.pop_front();
    }
  }
}

impl<'a> PacketBuf
{
  // TODO: make it not iterator through all the vecs every time
  #[inline]
  pub fn iter(&'a self) -> ViewIter<'a>
  {
    ViewIter {
      expected_seq: self.tail_seq,
      tagged_iter:  self.data.iter().peekable(),
    }
  }

  // TODO: make it not iterator through all the vecs every time
  #[inline]
  pub fn consume_iter(&'a mut self) -> ConsumeIter<'a>
  {
    ConsumeIter(self)
  }
}

#[cfg(test)]
mod test
{
  use std::num::Int;

  use super::super::PacketBuf as PacketBuf_T;
  //use super::PacketBuf;
  use super::*;

  #[test]
  fn iter_empty() {
    let vb: PacketBuf = PacketBuf_T::new(0);
    let mut iter = vb.iter();
    assert_eq!(iter.next(), None);
  }

  #[test]
  fn one_buf() {
    let mut vb: PacketBuf = PacketBuf_T::new(0);

    vb.add_slice(0, [1,5,4,3].as_slice());

    let mut iter = vb.iter();

    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(5));
    assert_eq!(iter.next(), Some(4));
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), None);
  }

  #[test]
  fn many_buf() {
    let mut vb: PacketBuf = PacketBuf_T::new(0);

    vb.add_slice(0, [1].as_slice());
    vb.add_slice(1, [3,2].as_slice());
    vb.add_slice(3, [6,5].as_slice());

    let mut iter = vb.iter();

    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(6));
    assert_eq!(iter.next(), Some(5));
    assert_eq!(iter.next(), None);
  }

  #[test]
  fn overlapping_buf() {
    let mut vb: PacketBuf = PacketBuf_T::new(0);

    vb.add_slice(0, [1,3,2,4].as_slice());
    vb.add_slice(2, [2,4,3,5].as_slice());
    vb.add_slice(4, [3,5,4,6].as_slice());

    let mut iter = vb.iter();

    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(4));
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), Some(5));
    assert_eq!(iter.next(), Some(4));
    assert_eq!(iter.next(), Some(6));
    assert_eq!(iter.next(), None);
  }

  #[test]
  fn wrapping_buf() {
    let u32_max: u32 = Int::max_value();

    let mut vb: PacketBuf = PacketBuf_T::new(u32_max - 3);

    vb.add_slice(u32_max - 3, [1,3,2,4].as_slice());
    vb.add_slice(u32_max - 1, [2,4,3,5].as_slice());
    vb.add_slice(u32_max + 1, [3,5,4,6].as_slice());

    let mut iter = vb.iter();

    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(4));
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), Some(5));
    assert_eq!(iter.next(), Some(4));
    assert_eq!(iter.next(), Some(6));
    assert_eq!(iter.next(), None);
  }

  #[test]
  fn mut_iter() {
    let u32_max: u32 = Int::max_value();

    let mut vb: PacketBuf = PacketBuf_T::new(u32_max - 3);

    vb.add_slice(u32_max - 3, [1,3,2,4].as_slice());
    vb.add_slice(u32_max - 1, [2,4,3,5].as_slice());
    vb.add_slice(u32_max + 1, [3,5,4,6].as_slice());

    {
      let mut iter = vb.consume_iter();

      assert_eq!(iter.next(), Some(1));
      assert_eq!(iter.next(), Some(3));
      assert_eq!(iter.next(), Some(2));
      assert_eq!(iter.next(), Some(4));
    }
    {
      let mut iter = vb.consume_iter();
      assert_eq!(iter.next(), Some(3));
      assert_eq!(iter.next(), Some(5));
      assert_eq!(iter.next(), Some(4));
      assert_eq!(iter.next(), Some(6));
      assert_eq!(iter.next(), None);
    }
  }
}
