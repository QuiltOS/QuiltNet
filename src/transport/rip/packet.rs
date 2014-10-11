use std::io::{
  BufReader,
  BufWriter,
  MemWriter,
  SeekSet,

  IoError,
  IoResult,
};
use std::mem::{transmute, size_of};

#[repr(packed)]
pub struct Entry {
  pub cost:    u32,
  pub address: u32,
}

#[repr(u16)]
#[repr(packed)]
pub enum Packet<Arr> {
  Request,
  Response(Arr),
}

pub fn parse<'a>(buf: &'a [u8]) -> Result<Packet<&'a [Entry]>, ()> {
  parse_helper(buf).map_err(|_| ())
}

fn parse_helper<'a>(buf: &'a [u8]) -> IoResult<Packet<&'a [Entry]>> {
  let mut r = BufReader::new(buf);
  match try!(r.read_be_u16()) {
    0 => Ok(Request),
    1 => {
      let count = try!(r.read_be_u16());
      // ought to be static
      let hdr_len: uint = size_of::<u16>() * 2;
      if buf.len() < size_of::<Entry>() + hdr_len {
        return Err(IoError::last_error()); // some random error
      }
      let entries: &'a[Entry] = unsafe {
        let s: &'a[Entry] = transmute(buf[hdr_len..]);
        s[..count as uint + 1] // 2 dots in exclusive
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
