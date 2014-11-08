use std::cmp;
use std::slice::bytes::copy_memory;

struct RingBuf {
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
      data : Vec::from_fn(size, |n| 0),
    }   
  }

  fn window_size(&self) -> uint {
    return self.head - self.tail;
  }

  // Reads as many bytes as possible into buf, returns number of bytes read
  pub fn read(&mut self, buf: &mut [u8]) -> uint {
    
    // Number of bytes we're going to copy
    let n = cmp::min(self.window_size(), buf.len());

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
    let n = cmp::min(len - self.window_size(), buf.len());
    println!("n: {}", n);
    

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
      println!("straight copy");

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
  fn t1(){
    let mut ring = RingBuf::new(10);
    let mut buf = [0u8, 0u8];
    println!("writing");
    println!("data: {}", ring.data);
    assert!(ring.write([1u8, 1u8]) == 2);
    println!("written");
    assert!(ring.read(buf) == 2);
    assert!(buf == [1u8,1u8]);
  }
}
