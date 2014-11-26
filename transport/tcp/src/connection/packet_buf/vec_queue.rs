use std::slice::Items;
use std::cmp;
use std::iter::{
  mod,

  Chain,
  Filter,
  FilterMap,
  FlatMap,
  Map,
  Rev,
  Scan,
  Skip,
  TakeWhile,
  Zip,
};
use std::num::Int;

use ring_buf::{mod, RingBuf};


#[deriving(PartialEq, Eq, Show)]
struct Tagged {
  seq:    u32,
  offset: uint,
  buf:    Vec<u8>,
}


impl PartialOrd for Tagged {
  fn partial_cmp(&self, other: &Tagged) -> Option<Ordering> {
    // backwards on purpose!
    // that way vec.pop() works
    other.seq.partial_cmp(&self.seq)
  }
}


impl Tagged
{
  #[inline]
  fn len(&self) -> uint
  {
    self.buf.len() - self.offset
  }

  #[inline]
  fn head(&self) -> u64
  {
    self.seq as u64 + self.len() as u64
  }

  #[inline]
  fn tail(&self) -> u64
  {
    self.seq as u64
  }

  #[inline]
  fn as_slice<'a>(&'a self) -> &'a [u8]
  {
    self.buf.as_slice()[self.offset..]
  }
}


#[deriving(Show)]
pub struct PacketBuf {
  tail_seq: u32,
  ind:      uint,
  data:     Vec<Tagged>,
}


impl super::PacketBuf for PacketBuf
{
  fn new(init_seq_num: u32) -> PacketBuf {
    PacketBuf {
      tail_seq: init_seq_num,
      ind:      0,
      data:     vec!(),
    }
  }

  fn get_next_consume_seq(&self) -> u32 { self.tail_seq }

  fn add_slice(&mut self, seq_num: u32, buf: &[u8]) -> u32
  {
    self.add_vec(seq_num, buf.to_vec(), 0)
  }

  fn add_vec(&mut self, seq_num: u32, vec: Vec<u8>, start_off: uint) -> u32
  {
    let u32_max: u32 = Int::max_value();

    let node = Tagged {
      seq:    seq_num,
      offset: start_off,
      buf:    vec,
    };
    assert!(node.len() < u32_max as uint);


    let ind = self.find_index(&node);

    // will make room
    self.data.insert(ind, node);

    // verify again after insert
    self.verify_index(ind);

    (self.data.len() - start_off) as u32 // always accept everything!
  }
}


impl PacketBuf
{
  #[inline]
  fn find_index(&self, node: &Tagged) -> uint
  {
    use std::slice::{Found, NotFound};

    let ind = self.data.binary_search(|prob| prob.partial_cmp(node).unwrap());
    let i = match ind { Found(i) => i, NotFound(i) => i, };

    self.verify_index(i);

    i
  }

  fn verify_index(&self, i: uint)
  {
    if let (Some(sm), Some(lg)) = (self.data.get(i + 1), self.data.get(i)) {
      verify_adjacent(sm, lg);
    }
    if let (Some(sm), Some(lg)) = (self.data.get(i), self.data.get(i - 1)) {
      verify_adjacent(sm, lg);
    }
  }
}


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
  let wrap   = sm.head() >= (lg.tail() + u32_max as u64);
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
  let wrap   = sm.head() <= (lg.head() + u32_max as u64);
  ! (normal || wrap)
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


pub type Fine<'a, I> = FlatMap<'a, (&'a Tagged, &'a Tagged), I, Items<'a, u8> >;
#[inline]
fn fine_iter<'a, I>(iter: I) -> Fine<'a, I>
  where I: Iterator<Pair<'a>>
{
  iter.flat_map(|(cur, next)| {
    let to = if cur.seq < next.seq {
      (next.seq - cur.seq) as uint
    } else {
      let u32_max: u32 = Int::max_value();
      (u32_max as u64 + next.seq as u64 - cur.seq as u64) as uint
    };

    cur.as_slice()[..to].iter()
  })
}


pub type View      <'a> = Fine<'a, Finite<'a, Double<'a, Cycle<'a>>>>;
pub type Consume   <'a> = View<'a>;


impl<'a>  PacketBuf
{
  #[inline]
  fn iter(&'a self) -> View<'a>
  {
    let iter = cyclic_corse_iter(&self.data, self.ind);
    fine_iter(finite_iter(double_iter(iter)))
  }

  #[inline]
  fn consume_iter(&'a mut self) -> View<'a>
  {
    let iter = cyclic_corse_iter(&self.data, self.ind);
    fine_iter(finite_iter(double_iter(iter)))
  }
}
