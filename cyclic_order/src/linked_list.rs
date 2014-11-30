use std::collections::dlist::{
  DList,
  ListInsertion,
};


// A priority queue to store cuts of a cycle
// Currently basically extension methods on DList
pub trait CutQueue<T> where T: ::CyclicOrd
{
  fn insert_cyclic(&mut self, T);
}

// Does not rotate link list
impl<T> CutQueue<T> for DList<T> where T: ::CyclicOrd
{
  fn insert_cyclic(&mut self, elt: T)
  {
    let mut it = self.iter_mut();

    // will be used to define the cut
    match it.next() {
      None        => (), // empty list, order does not matter
      Some(first) => loop {
        match it.peek_next() {
          None      => break,
          Some(cur) => if
            first.is_clockwise(&elt, cur) ||
            *first == elt || elt == *cur
          { break }
        }
        it.next();
      }
    };

    it.insert_next(elt);
  }
}


#[cfg(test)]
mod test
{
  use CyclicOrd;
  use CyclicOrdering::*;

  use super::CutQueue;
  use std::collections::DList;

  use quickcheck::TestResult;

  #[test]
  fn no_wrap() {
    let mut l = DList::new();
    l.insert_cyclic(0u8);
    l.insert_cyclic(3);
    l.insert_cyclic(1);
    l.insert_cyclic(4);

    let mut it = l.iter().map(|x| *x);

    assert_eq!(it.next(), Some(0));
    assert_eq!(it.next(), Some(1));
    assert_eq!(it.next(), Some(3));
    assert_eq!(it.next(), Some(4));
    assert_eq!(it.next(), None);
  }

  #[test]
  fn wrap() {
    let mut l = DList::new();
    l.insert_cyclic(5u8);
    l.insert_cyclic(3);
    l.insert_cyclic(1);
    l.insert_cyclic(4);
    l.insert_cyclic(255);
    l.insert_cyclic(0);

    let mut it = l.iter().map(|x| *x);

    assert_eq!(it.next(), Some(5));
    assert_eq!(it.next(), Some(255));
    assert_eq!(it.next(), Some(0));
    assert_eq!(it.next(), Some(1));
    assert_eq!(it.next(), Some(3));
    assert_eq!(it.next(), Some(4));
    assert_eq!(it.next(), None);
  }

  #[quickcheck]
  fn random_insertion(elems: Vec<u8>) -> TestResult
  {
    let mut l = DList::new();
    for n in elems.iter() {
      l.insert_cyclic(*n);
    }

    if l.len() < 3 { return TestResult::discard(); }

    let mut it = l.iter();

    let mut e1 = it.next().unwrap();
    let mut e2 = it.next().unwrap();
    let mut e3 = it.next().unwrap();

    loop {
      if e1.cyclic_cmp(e2, e3) == CounterClockwise {
        return TestResult::error(
          format!("triple: [{}, {}, {}]",e1, e2, e3).as_slice())
      };

      let e4 = match it.next() {
        None    => return TestResult::passed(),
        Some(e) => e,
      };

      e1 = e2;
      e2 = e3;
      e3 = e4;
    }
  }
}
