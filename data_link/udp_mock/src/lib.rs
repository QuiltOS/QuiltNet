#![feature(box_syntax)]
#![feature(unboxed_closures)]
#![feature(question_mark)]


#[macro_use]
extern crate log;

#[macro_use]
extern crate misc;
extern crate interface as dl;

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::io;
use std::net::{
  UdpSocket,
  SocketAddr,
  SocketAddrV4,
  Ipv4Addr,
  ToSocketAddrs,
};
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

use misc::interface as root;

#[cfg(test)]
mod test;

const RECV_BUF_SIZE: usize = 64 * 1024;

type SharedHandlerMap<'a> = Arc<RwLock<HashMap<SocketAddr,
                                               (bool, dl::Handler<'a>)>>>;

/// The backing listening socket / read loop for a bunch of UDP-backed mock link
/// neighbors
pub struct Listener<'a> {
  socket:   UdpSocket,
  handlers: SharedHandlerMap<'a>,
}

impl Listener<'static>
{
  pub fn new<A>(listen_addr: A, num_threads: usize) -> io::Result<Listener<'static>>
    where A: ToSocketAddrs
  {
    assert!(num_threads > 0);

    let socket = UdpSocket::bind(listen_addr)?;

    let handlers: SharedHandlerMap = Arc::new(RwLock::new(HashMap::new()));

    for _ in 0..num_threads {
      let socket   = socket.try_clone()?;
      let handlers = handlers.clone();
      thread::spawn(move || {
        let mut buf: [u8; RECV_BUF_SIZE] = unsafe { std::mem::uninitialized() };
        loop {
          // TODO: someway to kill task eventually
          match socket.recv_from(&mut buf[..]) {
            Err(e) => {
              // maybe it will work next time...
              debug!("OS error when trying to wait for packet: {}", e);
            },
            Ok((len, src_addr)) => match handlers.read().unwrap().get(&src_addr) {
              None          => continue, // drop that packet!
              Some(&(is_enabled, ref on_recv)) => {
                debug!("Received packet");
                if is_enabled {
                  debug!("Virtual Interface is enabled");
                  let args = buf[..len].to_vec();
                  (**on_recv)(args);
                } else {
                  debug!("Virtual Interface is not enabled, dropping packet");
                }
              },
            }
          };
          // )
        }
      });
    }

    Ok(Listener {
      socket:   socket,
      handlers: handlers,
    })
  }

  pub fn new_loopback(num_threads: usize) -> io::Result<(Self, SocketAddr)> {
    // "port 0" is wildcard (port number is dynamically assigned)
    let mut addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127,0,0,1), 0));
    let listener = Listener::new(addr, num_threads)?;
    listener.socket.connect(addr)?;
    // resolve actual port
    addr = listener.socket.local_addr()?;
    debug!("made listener with addr: {}", addr);
    Ok((listener, addr))
  }
}

impl<'a> Listener<'a>
{
  pub fn try_clone(&self) -> io::Result<Listener<'a>> {
    Ok(Listener {
      socket: self.socket.try_clone()?,
      handlers: self.handlers.clone(),
    })
  }
}


/// A mock link layer interface made from UDP
pub struct Interface<'a> {
  listener:    Listener<'a>,
  remote_addr: SocketAddr,
  cached_status: bool,
}


impl<'a> Interface<'a> {
  pub fn new(listener:    &Listener<'a>,
             remote_addr: SocketAddr,
             on_recv:     dl::Handler<'a>) -> Interface<'a>
  {
    listener.handlers.write().unwrap().insert(remote_addr, (true, on_recv));

    Interface {
      listener:      listener.try_clone().unwrap(),
      remote_addr:   remote_addr,
      cached_status: true,
    }
  }
}

impl<'a> root::Interface for Interface<'a> {
  type Error = io::Error;
}

impl<'a> dl::Interface<'a> for Interface<'a> {
  fn send(&self, packet: dl::Packet) -> dl::Result<(), Self::Error> {
    if self.cached_status == false {
      Err(dl::Error::Disabled)?;
    }
    let sent = self.listener
      .socket.try_clone()?
      .send_to(&packet[..], self.remote_addr)?;
    if sent != packet.len() {
      return Err(From::from(io::Error::new(
        io::ErrorKind::WriteZero,
        "The packet could not be sent in whole")));
    } else {
      Ok(())
    }
  }

  fn update_recv_handler<'b>(&'b self, on_recv: dl::Handler<'a>)
    where 'a: 'b
  {
    self.listener.handlers.write().unwrap()
      .insert(self.remote_addr, (true, on_recv));
  }

  fn enable(&mut self) {
    let mut map = self.listener.handlers.write().unwrap();
    self.cached_status = true;
    match map.entry(self.remote_addr) {
      Vacant(_) => panic!("udp mock interface should already have entry in table"),
      Occupied(mut entry) => {
        entry.get_mut().0 = true;
      },
    }
  }

  fn disable(&mut self) {
    let mut map = self.listener.handlers.write().unwrap();
    self.cached_status = false;
    match map.entry(self.remote_addr) {
      Vacant(_) => panic!("udp mock interface should already have entry in table"),
      Occupied(mut entry) => {
        entry.get_mut().0 = false;
      },
    }
  }

  fn get_status(&self) -> bool {
    self.cached_status
  }

}
