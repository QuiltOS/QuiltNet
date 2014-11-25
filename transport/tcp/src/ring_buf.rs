use std::cmp;
use std::slice::bytes::copy_memory;
use std::slice::Items;
use std::iter::{Chain, Map, Scan};

#[deriving(Show)]
pub struct RingBuf {
  tail: uint, // Back of unconsumed, valid data  -- oldest byte written
  head: uint, // Front of unconsumed, valid data -- next byte to write
  data: Vec<u8>,
}
// head == tail is declared to be empty


impl RingBuf
{
  // Returns new Ring Buffer of given (fixed) size
  pub fn new(size: uint) -> RingBuf {
    RingBuf {
      tail: 0,
      head: 0,
      data: Vec::from_fn(size + 1, |n| 0),
    }
  }

  #[inline]
  fn check_invariants(&self) {
    // always at least one byte in ring buf
    assert!(self.data.len() > 0);
    // head and tail always point to a valid byte
    assert!(self.tail < self.data.len());
    assert!(self.head < self.data.len());

    // always at least one empty byte -- but you can't use it
    assert!(self.readable_len() < self.data.len());
  }


  #[inline]
  pub fn writable_len(&self) -> uint
  {
    self.check_invariants();
    self.data.len() - self.readable_len() - 1
  }

  /// The number of readable/valid bytes
  #[inline]
  pub fn readable_len(&self) -> uint
  {
    if self.tail <= self.head {
      self.head - self.tail
    } else {
      self.data.len() - self.tail + self.head
    }
  }

  // Reads as many bytes as possible into buf, returns number of bytes read
  pub fn read(&mut self, buf: &mut [u8]) -> uint
  {
    self.check_invariants();

    // Number of bytes we're going to copy
    let n = cmp::min(self.readable_len(), buf.len());
    debug!("read: n: {}, ws: {}", n, self.readable_len());

    // Head of slice we're reading from, wrapped around
    let read_head = (self.tail + n) % self.data.len();

    // If we need to wrap around
    if self.tail > read_head
    {
      let first_slice_len = self.data.len() - self.tail;

      // read until end of vec into first slice of buf
      copy_memory(buf[mut ..first_slice_len], self.data[self.tail..]);

      // read from start until head into second slice of buf
      copy_memory(buf[mut first_slice_len..n], self.data[0..read_head]);
    }
    else
    {
      // Copy straight until head
      copy_memory(buf, self.data[self.tail..read_head]);
    }

    // Move tail to reflect consumption
    self.tail = read_head;

    // Return # bytes read
    self.check_invariants();
    n
  }

  // Writes as many bytes as possible from buf, returns number of bytes written
  pub fn write(&mut self, buf: &[u8]) -> uint
  {
    self.check_invariants();

    let len = self.data.len();

    // Number of bytes we're going to copy
    // NOTE: subtract 1 to avoid writing full array - we can't disambiguate full/empty!
    let n = cmp::min(self.writable_len(), buf.len());
    //println!("write: n: {}, ws: {}", n, self.readable_len());


    // Head of slice we're writing into, wrapped around
    let write_head = (self.head + n) % len;

    // If we need to wrap around
    if self.tail > write_head
    {
      let first_slice_len = len - self.head;

      // read until end of vec into first slice of buf
      copy_memory(self.data[mut self.head..len], buf[0..first_slice_len]);

      // read from start until head into second slice of buf
      copy_memory(self.data[mut ..write_head], buf[first_slice_len..n] );
    }
    else
    {
      // Copy straight until head
      copy_memory(self.data[mut self.head..write_head], buf);
    }

    // Move head to front of newly written data
    self.head = write_head;

    // Return # bytes read
    self.check_invariants();
    n
  }
}

impl<'a> RingBuf
{
  #[inline]
  pub fn iter<'a>(&'a self) -> View<'a>
  {
    raw_make_iter(&self.data, self.head, self.tail)
  }

  #[inline]
  pub fn consume_iter<'a>(&'a mut self) -> Consume<'a>
  {
    let len: uint = self.data.len();

    // TODO close over len instead
    let inc: |&mut (&mut uint, uint), u8|:'a -> Option<u8> = |st, b| {
      *st.0 = (*st.0 + 1) % st.1;
      Some(b)
    };

    raw_make_iter(&self.data, self.head, self.tail)
      .scan((&mut self.tail, len), inc)
  }
}


pub type View<'a>    = Map<'a, &'a u8, u8, Chain<Items<'a, u8>, Items<'a, u8>>>;
pub type Consume<'a> = Scan<'a, u8, u8, View<'a>, (&'a mut uint, uint)>;


// for finer-grain borrowing
#[inline]
fn raw_make_iter<'a>(data: &'a Vec<u8>,
                     head: uint,
                     tail: uint) -> View<'a>
{
  let len = data.len();

  if head < tail
  {
    // we need to wrap around
    data[tail..]
      .iter()
      .chain(data[..head].iter())
  }
  else
  {
    // continuous (including empty)

    // we need to chain so that the types are the same
    data[tail..head]
      .iter()
      .chain(data[head..head].iter())
  }
  .map(|x| *x)
}


#[cfg(test)]
mod test
{
  use super::RingBuf;

  #[test]
  fn empty_readable_lens() {
    let mut ring = RingBuf::new(0);
    assert_eq!(ring.readable_len(), 0);
  }

  #[test]
  fn single_valid_byte() {
    let mut ring = RingBuf::new(1);
    assert_eq!(ring.write([1].as_slice()), 1);
    assert_eq!(ring.readable_len(), 1);
  }

  #[test]
  fn simple(){
    let mut ring = RingBuf::new(10);
    let mut buf = [0u8, 0u8];
    assert_eq!(ring.write([1, 1].as_slice()), 2);
    println!("after write: {}", ring);
    assert_eq!(ring.read(buf.as_mut_slice()), 2);
    println!("after read: {}", ring);
    assert_eq!(buf, [1,1]);
  }

 #[test]
  fn slightly_more_complex(){
    let mut ring = RingBuf::new(10);
    ring.head = 8;
    ring.tail = 8;
    let mut buf = [0u8, 0u8, 0u8, 0u8, 0u8];
    assert_eq!(ring.write([1, 1, 2, 3, 4].as_slice()), 5);
    println!("after write: {}", ring);
    assert_eq!(ring.read(buf.as_mut_slice()), 5);
    println!("after read: {}", ring);
    assert_eq!(buf, [1,1, 2, 3, 4]);
  }

  #[test]
  fn wrap(){
    let mut ring = RingBuf::new(4);
    let mut buf  = [0u8,0u8,0u8,0u8];
    assert_eq!(ring.write([1,2,3].as_slice()), 3);
    println!("After write1: {}", ring);
    assert_eq!(ring.read(buf.as_mut_slice()), 3);
    println!("After read1: {}", ring);
    assert_eq!(buf, [1, 2, 3, 0]);

    assert_eq!(ring.write([4, 5, 6, 7].as_slice()), 4);
    println!("After write2: {}", ring);
    assert_eq!(ring.read(buf.as_mut_slice()), 4);
    println!("After read2: {}", ring);
    println!("buf: {}", buf.as_slice());
    assert_eq!(buf, [4, 5, 6, 7])
  }

  #[test]
  fn simple_non_consuming() {
    let mut ring = RingBuf::new(4);

    assert_eq!(ring.write([1,2,3,4].as_slice()), 4);

    assert_eq!(ring.tail, 0);
    assert_eq!(ring.head, 4);
    assert_eq!(ring.readable_len(), 4);

    ring.check_invariants();

    let mut iter = ring.iter();

    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(3));
    assert_eq!(iter.next(), Some(4));
    assert_eq!(iter.next(), None);

    ring.check_invariants();

    assert_eq!(ring.tail, 0);
    assert_eq!(ring.head, 4);
    assert_eq!(ring.readable_len(), 4);
  }

  #[test]
  fn simple_consuming() {
    let mut ring = RingBuf::new(4);
    assert_eq!(ring.write([1,2,3,4].as_slice()), 4);

    assert_eq!(ring.tail, 0);
    assert_eq!(ring.head, 4);
    assert_eq!(ring.readable_len(), 4);

    ring.check_invariants();

    {
      let mut iter = ring.consume_iter();

      assert_eq!(iter.next(), Some(1));
      assert_eq!(iter.next(), Some(2));
      assert_eq!(iter.next(), Some(3));
      assert_eq!(iter.next(), Some(4));
      assert_eq!(iter.next(), None);
    }

    ring.check_invariants();

    assert_eq!(ring.tail, 4);
    assert_eq!(ring.head, 4);
    assert_eq!(ring.readable_len(), 0);
  }
}
