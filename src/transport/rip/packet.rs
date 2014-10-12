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

#[deriving(PartialEq, PartialOrd, Eq, Ord,
           Clone, Show)]
#[repr(packed)]
pub struct Entry {
  pub cost:    u32,
  pub address: u32,
}

#[deriving(PartialEq, PartialOrd, Eq, Ord,
           Clone, Show)]
#[repr(u16)]
#[repr(packed)]
pub enum Packet<Arr> {
  Request,
  Response(Arr),
}

pub fn parse_ip(bits: u32) -> IpAddr {
  let [a, b, c, d]: [u8, ..4] = unsafe { transmute(bits) };
  Ipv4Addr(a, b, c, d)
}

pub fn write_ip(addr: IpAddr) -> u32{
  match addr {
    Ipv4Addr(a, b, c, d) => unsafe { transmute([a, b, c, d]) },
    _                    => fail!("no ipv6 yet"),
  }
}

pub fn parse<'a>(buf: &'a [u8]) -> Result<Packet<&'a [Entry]>, ()> {
  parse_helper(buf).map_err(|_| ())
}

fn parse_helper<'a>(buf: &'a [u8]) -> IoResult<Packet<&'a [Entry]>> {
  let mut r = BufReader::new(buf);
  match try!(r.read_be_u16()) {
    1 => Ok(Request),
    2 => {
      let count = try!(r.read_be_u16());
      // ought to be static
      let hdr_len: uint = size_of::<u16>() * 2;
      let body_len: uint = size_of::<Entry>() * count as uint;

      let entries: &'a[Entry] = match buf.len().cmp(&(body_len + hdr_len)) {
        Less    => return Err(IoError::last_error()), // some random error
        Equal   => {
          unsafe { transmute(buf[hdr_len..]) }
        },
        Greater => {
          println!("Rip: packet was too large");
          let s: &'a[Entry] = unsafe { transmute(buf[hdr_len..]) };
          s[..count as uint + 1] // 2 dots in exclusive
        },
      };

      Ok(Response(entries))
    },
    _ => Err(IoError::last_error()), // some random error
  }
}

pub fn write<'a, I>(packet: Packet<I>) -> proc(&Vec<u8>):'a -> IoResult<()>
  where I: Iterator<Entry> + 'a
{
  proc(vec) {
    let packet = packet;
    // MemWriter is just a newtype
    let m: &mut MemWriter = unsafe { transmute(vec) };
    match packet {
      Request => {
        try!(m.write_be_u16(0));
        try!(m.write_be_u16(0));
      },
      Response(mut iter) => {
        try!(m.write_be_u16(1));
        try!(m.write_be_u16(0xFAAF)); // place holder
        let mut count = 0;
        for Entry { cost, address } in iter {
          count += 1;
          try!(m.write_be_u32(cost));
          try!(m.write_be_u32(address));
        }
        // cast back, because previous cast was interpreted as move
        let vec2: &mut Vec<u8> = unsafe { transmute(m) };
        {
          let mut b = BufWriter::new(vec2.as_mut_slice());
          try!(b.seek(size_of::<u16>() as i64, SeekSet));
          try!(b.write_be_u16(count));
        }
      },
    }
    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::*;

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

}
