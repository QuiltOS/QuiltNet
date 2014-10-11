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
enum Packet<Arr> {
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
            let HDR_LEN: uint = size_of::<u16>() * 2;
            if buf.len() < size_of::<Entry>() + HDR_LEN {
                return Err(IoError::last_error()); // some random error
            }
            let mut entries: &'a[Entry] = unsafe {
                let s: &'a[Entry] = transmute(buf[HDR_LEN..]);
                s[..count as uint + 1] // 2 dots in exclusive
            };
            Ok(Response(entries))
        },
        _ => Err(IoError::last_error()), // some random error
    }
}

pub fn write<I>(packet: Packet<I>) -> IoResult<Vec<u8>>
    where I: Iterator<Entry>
{
    let mut m = MemWriter::new();
    match packet {
        Request => {
            try!(m.write_be_u16(0));
            try!(m.write_be_u16(0));
            Ok(m.unwrap())
        },
        Response(mut iter) => {
            try!(m.write_be_u16(1));
            try!(m.write_be_u16(0xFAAF)); // place holder
            let mut count = 0;
            for Entry { cost, address } in iter {
                count += 1;
                try!(m.write_be_u32(cost));
                try!(m.write_be_u32(cost));
            }
            let mut v = m.unwrap();
            {
                let mut b = BufWriter::new(v.as_mut_slice());
                try!(b.seek(size_of::<u16>() as i64, SeekSet));
                try!(b.write_be_u16(count));
            }
            Ok(v)
        },
    }

}
