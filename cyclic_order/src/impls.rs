use std::num::UnsignedInt;

use super::*;
use super::CyclicOrdering::{
  Clockwise,
  CounterClockwise,
  Degenerate
};

impl<N> PartialCyclicOrd for N where N: UnsignedInt + Ord
{
  #[inline]
  fn is_clockwise(&self, them: &N, other: &N) -> bool
  {
    self.cyclic_cmp(them, other) == Clockwise
  }
}

impl<N> CyclicOrd for N where N: UnsignedInt + Ord
{
  #[inline]
  fn cyclic_cmp(&self, them: &N, other: &N) -> CyclicOrdering
  {
    if
      (self  < them  && them  < other) ||
      (them  < other && other < self)  ||
      (other < self  && self  < them)
    {
      CyclicOrdering::Clockwise
    }
    else if
      (self  > them  && them  > other) ||
      (them  > other && other > self)  ||
      (other > self  && self  > them)
    {
      CyclicOrdering::CounterClockwise
    }
    else { Degenerate }
  }
}

#[cfg(test)]
mod partial_test {
  use partial_axioms::*;
  
  #[quickcheck] fn u8_cyclicity(a: u8, b: u8, c: u8) -> bool { cyclicity(&a, &b, &c) }
  #[quickcheck] fn u16_cyclicity(a: u16, b: u16, c: u16) -> bool { cyclicity(&a, &b, &c) }
  #[quickcheck] fn u32_cyclicity(a: u32, b: u32, c: u32) -> bool { cyclicity(&a, &b, &c) }
  #[quickcheck] fn u64_cyclicity(a: u64, b: u64, c: u64) -> bool { cyclicity(&a, &b, &c) }
  #[quickcheck] fn usize_cyclicity(a: usize, b: usize, c: usize) -> bool { cyclicity(&a, &b, &c) }

  #[quickcheck] fn u8_antisymmetry(a: u8, b: u8, c: u8) -> bool { antisymmetry(&a, &b, &c) }
  #[quickcheck] fn u16_antisymmetry(a: u16, b: u16, c: u16) -> bool { antisymmetry(&a, &b, &c) }
  #[quickcheck] fn u32_antisymmetry(a: u32, b: u32, c: u32) -> bool { antisymmetry(&a, &b, &c) }
  #[quickcheck] fn u64_antisymmetry(a: u64, b: u64, c: u64) -> bool { antisymmetry(&a, &b, &c) }
  #[quickcheck] fn usize_antisymmetry(a: usize, b: usize, c: usize) -> bool { antisymmetry(&a, &b, &c) }

  #[quickcheck] fn u8_transitivity(a: u8, b: u8, c: u8, d: u8) -> bool { transitivity(&a, &b, &c, &d) }
  #[quickcheck] fn u16_transitivity(a: u16, b: u16, c: u16, d: u16) -> bool { transitivity(&a, &b, &c, &d) }
  #[quickcheck] fn u32_transitivity(a: u32, b: u32, c: u32, d: u32) -> bool { transitivity(&a, &b, &c, &d) }
  #[quickcheck] fn u64_transitivity(a: u64, b: u64, c: u64, d: u64) -> bool { transitivity(&a, &b, &c, &d) }
  #[quickcheck] fn usize_transitivity(a: usize, b: usize, c: usize, d: usize) -> bool { transitivity(&a, &b, &c, &d) }
}

#[cfg(test)]
mod total_test {
  use total_axioms::*;
  
  #[quickcheck] fn u8_cyclicity(a: u8, b: u8, c: u8) -> bool { cyclicity(&a, &b, &c) }
  #[quickcheck] fn u16_cyclicity(a: u16, b: u16, c: u16) -> bool { cyclicity(&a, &b, &c) }
  #[quickcheck] fn u32_cyclicity(a: u32, b: u32, c: u32) -> bool { cyclicity(&a, &b, &c) }
  #[quickcheck] fn u64_cyclicity(a: u64, b: u64, c: u64) -> bool { cyclicity(&a, &b, &c) }
  #[quickcheck] fn usize_cyclicity(a: usize, b: usize, c: usize) -> bool { cyclicity(&a, &b, &c) }

  #[quickcheck] fn u8_antisymmetry(a: u8, b: u8, c: u8) -> bool { antisymmetry(&a, &b, &c) }
  #[quickcheck] fn u16_antisymmetry(a: u16, b: u16, c: u16) -> bool { antisymmetry(&a, &b, &c) }
  #[quickcheck] fn u32_antisymmetry(a: u32, b: u32, c: u32) -> bool { antisymmetry(&a, &b, &c) }
  #[quickcheck] fn u64_antisymmetry(a: u64, b: u64, c: u64) -> bool { antisymmetry(&a, &b, &c) }
  #[quickcheck] fn usize_antisymmetry(a: usize, b: usize, c: usize) -> bool { antisymmetry(&a, &b, &c) }

  #[quickcheck] fn u8_transitivity(a: u8, b: u8, c: u8, d: u8) -> bool { transitivity(&a, &b, &c, &d) }
  #[quickcheck] fn u16_transitivity(a: u16, b: u16, c: u16, d: u16) -> bool { transitivity(&a, &b, &c, &d) }
  #[quickcheck] fn u32_transitivity(a: u32, b: u32, c: u32, d: u32) -> bool { transitivity(&a, &b, &c, &d) }
  #[quickcheck] fn u64_transitivity(a: u64, b: u64, c: u64, d: u64) -> bool { transitivity(&a, &b, &c, &d) }
  #[quickcheck] fn usize_transitivity(a: usize, b: usize, c: usize, d: usize) -> bool { transitivity(&a, &b, &c, &d) }

  #[quickcheck] fn u8_totality(a: u8, b: u8, c: u8) -> bool { totality(&a, &b, &c) }
  #[quickcheck] fn u16_totality(a: u16, b: u16, c: u16) -> bool { totality(&a, &b, &c) }
  #[quickcheck] fn u32_totality(a: u32, b: u32, c: u32) -> bool { totality(&a, &b, &c) }
  #[quickcheck] fn u64_totality(a: u64, b: u64, c: u64) -> bool { totality(&a, &b, &c) }
  #[quickcheck] fn usize_totality(a: usize, b: usize, c: usize) -> bool { totality(&a, &b, &c) }

  #[quickcheck] fn u8_super_trait_cohesion(a: u8, b: u8, c: u8) -> bool { super_trait_cohesion(&a, &b, &c) }
  #[quickcheck] fn u16_super_trait_cohesion(a: u16, b: u16, c: u16) -> bool { super_trait_cohesion(&a, &b, &c) }
  #[quickcheck] fn u32_super_trait_cohesion(a: u32, b: u32, c: u32) -> bool { super_trait_cohesion(&a, &b, &c) }
  #[quickcheck] fn u64_super_trait_cohesion(a: u64, b: u64, c: u64) -> bool { super_trait_cohesion(&a, &b, &c) }
  #[quickcheck] fn usize_super_trait_cohesion(a: usize, b: usize, c: usize) -> bool { super_trait_cohesion(&a, &b, &c) }
}
