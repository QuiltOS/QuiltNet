use std::io::net::ip::{IpAddr, Ipv4Addr};
use std::io::{
  BufReader,
  BufWriter,
  MemWriter,
  SeekSet,

  IoError,
  IoResult,
};
use std::mem::{transmute, size_of};

use super::RIP_MAX_ENTRIES;

#[deriving(PartialEq, Eq, Clone, Show)]
#[repr(packed)]
pub struct Entry {
  pub cost:    u32,
  pub address: IpAddr,
}

#[deriving(PartialEq, PartialOrd, Eq, Ord,
           Clone, Show)]
#[repr(u16)]
#[repr(packed)]
pub enum Packet<Arr> {
  Request,
  Response(Arr),
}

#[inline]
pub fn parse_ip(&[a, b, c, d]: &[u8, ..4]) -> IpAddr {
  Ipv4Addr(a, b, c, d)
}

#[inline]
pub fn write_ip(addr: IpAddr) -> [u8, ..4] {
  match addr {
    Ipv4Addr(a, b, c, d) => [a, b, c, d],
    _                    => panic!("no ipv6 yet"),
  }
}

pub fn parse<'a>(buf: &'a [u8]) -> Result<Packet<Entries<'a>>, ()> {
  parse_helper(buf).map_err(|_| ())
}

pub type Entries<'a> = EntryIter<BufReader<'a>>;

struct EntryIter<R>(R);

impl<R> EntryIter<R> where R: Reader {

  fn next_helper(&mut self) -> IoResult<Entry> {
    let &EntryIter(ref mut r)      = self;
    let cost                   = try!(r.read_be_u32());
    let mut addr_buf: [u8, ..4] = [0, 0, 0, 0];
    try!(r.read(addr_buf.as_mut_slice()));
    let address                = parse_ip(&addr_buf);
    Ok(Entry { cost: cost, address: address })
  }
}

impl<R> Iterator<Entry> for EntryIter<R> where R: Reader
{
  fn next(&mut self) -> Option<Entry> {
    self.next_helper().ok()
  }
}

#[inline]
fn parse_helper<'a>(buf: &'a [u8]) -> IoResult<Packet<Entries<'a>>>
{
  let mut r = BufReader::new(buf);
  match try!(r.read_be_u16()) {
    1 => Ok(Request),
    2 => {
      let count = try!(r.read_be_u16());
      // ought to be static
      let hdr_len:  uint = size_of::<u16>() * 2;
      let body_len: uint = size_of::<u32>() * 2 * count as uint;

      match buf.len().cmp(&(body_len + hdr_len)) {
        Less    => return Err(IoError::last_error()), // some random error
        Greater => debug!("packet was too large"),
        Equal   => (),
      }

      Ok(Response(EntryIter(r)))
    },
    _ => Err(IoError::last_error()), // some random error
  }
}

pub fn write_request(vec: &Vec<u8>) -> IoResult<()>
{
  // MemWriter is just a newtype
  let m: &mut MemWriter = unsafe { transmute(vec) };
  try!(m.write_be_u16(1));
  try!(m.write_be_u16(0));
  Ok(())
}

// iterator must yield at least one entry, or assertion will fall
pub fn write_response<'a, I>(entries_iter: &'a mut I)
                             -> proc(&mut Vec<u8>):'a -> IoResult<()>
  where I: Iterator<Entry>
{
  proc(vec)
  {
    let entries_iter = entries_iter;
    let thus_far = vec.len();
    let mut count: u16 = 0;
    {
      // MemWriter is just a newtype
      let m: &mut MemWriter = unsafe { transmute(&*vec) };
      try!(m.write_be_u16(2));
      try!(m.write_be_u16(0x_FF_FF)); // place holder

      // this seems overly manual, except I need to remember number of times I
      // loop
      while count < RIP_MAX_ENTRIES {
        let Entry { cost, address } = match entries_iter.next() {
          Some(e) => e,
          None    => break,
        };
        count += 1;
        try!(m.write_be_u32(cost));
        try!(m.write(write_ip(address)));
      }
      assert!(count > 0); // we don't want to send empty response packets
    }
    {
      let mut b = BufWriter::new(vec.as_mut_slice());
      try!(b.seek((thus_far + size_of::<u16>()) as i64, SeekSet));
      debug!("fixing count ({}) when writing entries_iter", count);
      try!(b.write_be_u16(count));
    }
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  use std::io::net::ip::{IpAddr, Ipv4Addr};
  use std::io::{
    BufReader,
    BufWriter,
    MemWriter,
    SeekSet,

    IoError,
    IoResult,
  };

  #[test]
  fn parse_invalid() {
    assert!(parse(&[0]).is_err());
    assert!(parse(&[1]).is_err());
    assert!(parse(&[2]).is_err());

    assert!(parse(&[0,0]).is_err());
    assert!(parse(&[1,0]).is_err());
    assert!(parse(&[2,0]).is_err());

    assert!(parse(&[0,0]).is_err());
    assert!(parse(&[1,0,0]).is_err());
    assert!(parse(&[2,0,0,0]).is_err());

    assert!(parse(&[1,1,0]).is_err());
    assert!(parse(&[2,1,0,0]).is_err());
  }

  #[test]
  fn parse_request() {
    assert_eq!(parse(&[0,1]), Ok(Request));
  }

  #[test]
  fn parse_response() {
    let empty: [Entry, ..0] = [];
    assert_eq!(parse(&[0,2,0,0]), Ok(Response(empty.as_slice())));
  }

 #[test]
 fn write_request() {
   let msg: &[u8] = &[0,1,0,0];
   let vec = Vec::new();
   super::write_request(&vec).unwrap();
   assert_eq!(vec.as_slice(), msg);
 }

  #[test]
  fn write_response() {
    {
      let empty: [Entry, ..0] = [];
      let msg: &[u8] = &[0,2,0,0];

      let vec = Vec::new();
      write(Response(empty.iter().map(|x| *x)))(&vec).unwrap();
      assert_eq!(vec.as_slice(), msg);
    }
    {
      let entries = [Entry { cost: 5,  address: write_ip(Ipv4Addr(1,2,3,4)) },
                     Entry { cost: 16, address: write_ip(Ipv4Addr(5,4,3,2)) }];
      let msg: &[u8] = &[0,2,0,2,
                         0,0,0,5,  1,2,3,4,
                         0,0,0,16, 5,4,3,2];

      let vec = Vec::new();
      super::write_responce(Response(entries.iter().map(|x| *x)))(&vec).unwrap();
      assert_eq!(vec.as_slice(), msg);
    }
  }

}
