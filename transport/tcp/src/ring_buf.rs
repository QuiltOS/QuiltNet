use std::cmp;
use std::slice::bytes::copy_memory;

#[deriving(Show)]
pub struct RingBuf {
  tail : uint, // Back of unconsumed, valid data
  head : uint,     // Front of unconsumed, valid data
  data : Vec<u8>,
}

impl RingBuf {

  // Returns new Ring Buffer of given (fixed) size
  pub fn new(size: uint) -> RingBuf {
    RingBuf {
      tail : 0,
      head : 0,
      // add 1 slot to disambiguate full/empty
      data : Vec::from_fn(size + 1, |n| 0),
    }
  }

  fn window_size(&self) -> uint {
    if self.tail <= self.head {
      self.head - self.tail
    } else {
      self.data.len() - self.tail + self.head
    }
  }

  // Reads as many bytes as possible into buf, returns number of bytes read
  pub fn read(&mut self, buf: &mut [u8]) -> uint {

    // Number of bytes we're going to copy
    let n = cmp::min(self.window_size(), buf.len());
    //println!("read: n: {}, ws: {}", n, self.window_size());

    // Head of slice we're reading from, wrapped around
    let read_head = (self.tail + n) % self.data.len();

    // If we need to wrap around
    if self.tail > read_head {

      let first_slice_len = self.data.len() - self.tail;

      // read until end of vec into first slice of buf
      copy_memory(buf.slice_mut(0, first_slice_len), self.data[self.tail..]);

      // read from start until head into second slice of buf
      copy_memory(buf.slice_mut(first_slice_len, n), self.data[0..read_head]);

    } else {

      // Copy straight until head
      copy_memory(buf, self.data[self.tail..read_head]);

    }

    // Move tail to reflect consumption
    self.tail = read_head;

    // Return # bytes read
    n
  }

  // Writes as many bytes as possible from buf, returns number of bytes written
  pub fn write(&mut self, buf: &[u8]) -> uint {

    let len = self.data.len();

    // Number of bytes we're going to copy
    // NOTE: subtract 1 to avoid writing full array - we can't disambiguate full/empty!
    let n = cmp::min(len - self.window_size() - 1, buf.len());
    //println!("write: n: {}, ws: {}", n, self.window_size());


    // Head of slice we're writing into, wrapped around
    let write_head = (self.head + n) % len;

    // If we need to wrap around
    if self.tail > write_head {

      let first_slice_len = len - self.head;

      // read until end of vec into first slice of buf
      copy_memory(self.data.slice_mut(self.head, len), buf[0..first_slice_len]);

      // read from start until head into second slice of buf
      copy_memory(self.data.slice_mut(0, write_head), buf[first_slice_len..n] );

    } else {

      // Copy straight until head
      copy_memory(self.data.slice_mut(self.head, write_head), buf);

    }

    // Move head to front of newly written data
    self.head = write_head;

    // Return # bytes read
    n
  }
}

#[cfg(test)]
mod test {

  use super::RingBuf;

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

/*  #[test]
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
  */
}
