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

/*
#[inline]
fn verify_adjacent(sm: &Tagged, lg: &Tagged)
{
  // correct order
  assert!(sm.seq < lg.seq);

  // nobody contains the other
  //assert!(! ( sm.head() > lg.head() ) );
}


#[inline]
/// Stops when there is a discontinuity
fn no_gap(sm: &Tagged, lg: &Tagged) -> bool
{
  let u32_max: u32 = Int::max_value();

  let normal = sm.head() >= lg.tail();
  let wrap   = sm.head() >= (lg.tail() + u32_max as u64) && (sm.head() < sm.tail());
  normal || wrap
}


// TODO eventually we might want to prevent this from occuring in the first
// place. Easy enough in the normal case, harder to prevent in the wrapping case

#[inline]
/// Returns false when `lg` is wholly redundant. Expects no gap.
fn no_subsumption(sm: &Tagged, lg: &Tagged) -> bool
{
  let u32_max: u32 = Int::max_value();

  let normal = sm.head() <= lg.head();
  let wrap   = sm.head() <= (lg.head() + u32_max as u64) && (sm.head() < sm.tail());
  normal || wrap
}




// Can't take self for borrowing purposes

pub type Cycle<'a> = Skip<iter::Cycle<Rev<Items<'a, Tagged>>>>;
/// returns an infinite stream of slices
#[inline]
fn cyclic_corse_iter<'a>(v: &'a Vec<Tagged>, ind: uint) -> Cycle<'a>
{
  v.iter().rev().cycle().skip(ind)
}

pub type Double<'a, I> = Zip<I, I>;
#[inline]
fn double_iter<'a, I, T>(iter: I) -> Double<'a, I>
  where I: Clone + Iterator<T>
{
  let mut iter2 = iter.clone();
  iter2.next();

  iter.zip(iter2)
}

pub type Pair<'a> = (&'a Tagged, &'a Tagged);
pub type Finite<'a, I> = Filter<'a, Pair<'a>, TakeWhile<'a, Pair<'a>, I>>;
#[inline]
fn finite_iter<'a, I>(iter: I) -> Finite<'a, I>
  where I: Iterator<Pair<'a>>
{
  iter
    .take_while(|&(cur, next)| no_gap(cur, next))
    .filter(|&(cur, next)| no_subsumption(cur, next))
}
*/

//pub type Concat<'a, I> = FlatMap<'a, Pair<'a>, I, Map<'a, &'a u8, u8, Items<'a, u8> >>;

pub type Concat<'a, I, A> = Scan<'a, (A, u32), A, I, u32>;
#[inline]
fn concat<'a, I, A>(first_seq: u32, iter: I) -> Concat<'a, I, A>
  where I: Iterator<(A, u32)>
{
  iter.scan(first_seq, move |expected_seq, (byte, cur_seq)| {
    if *expected_seq == cur_seq {
      *expected_seq += 1;
      Some(byte)
    } else {
      None
    }
  })
}

pub type Immut<'a> = FlatMap<'a,
                             &'a Tagged,
                             dlist::Items<'a, Tagged>,
                             Zip<Map<'a, &'a u8, u8, slice::Items<'a, u8>>,
                             Counter<u32>>>;

pub type View   <'a> = Concat<'a, Immut<'a>, u8>;
pub type Consume<'a> = View<'a>;

impl<'a>  PacketBuf
{
  // TODO: make it not iterator through all the vecs every time
  #[inline]
  fn iter(&'a self) -> View<'a>
  {
    let chunks = self.data.iter();

    let numbered = chunks.flat_map(|node| {
      let slice = node.as_slice();
      slice.iter()
        .map(|x| *x)
        .zip(::std::iter::count(node.tail(), 1))
    });

    concat(self.tail_seq, numbered)
  }

  // TODO: make it not iterator through all the vecs every time
  #[inline]
  fn consume_iter(&'a mut self) -> Consume<'a>
  {
    self.iter()
  }
}

#[cfg(test)]
mod test
{
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
}
