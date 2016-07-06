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
  ToSocketAddrs,
};
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

use misc::interface as root;

#[cfg(test)]
mod test;

const RECV_BUF_SIZE: usize = 64 * 1024;

type SharedHandlerMap = Arc<RwLock<HashMap<SocketAddr,
                                           (bool, dl::Handler)>>>;

/// The backing listening socket / read loop for a bunch of UDP-backed mock link
/// neighbors
pub struct Listener {
  socket:   UdpSocket,
  handlers: SharedHandlerMap,
}

impl Listener
{
  pub fn new<A>(listen_addr: A, num_threads: usize) -> io::Result<Listener>
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

  pub fn try_clone(&self) -> io::Result<Listener> {
    Ok(Listener {
      socket: self.socket.try_clone()?,
      handlers: self.handlers.clone(),
    })
  }
}


/// A mock link layer interface made from UDP
pub struct Interface {
  listener:    Listener,
  remote_addr: SocketAddr,
  cached_status: bool,
}


impl Interface {
  pub fn new(listener:    &Listener,
             remote_addr: SocketAddr,
             on_recv:     dl::Handler) -> Interface
  {
    listener.handlers.write().unwrap().insert(remote_addr, (true, on_recv));

    Interface {
      listener:      listener.try_clone().unwrap(),
      remote_addr:   remote_addr,
      cached_status: true,
    }
  }
}

impl root::Interface for Interface {
  type Error = io::Error;
}

impl dl::Interface for Interface {
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

  fn update_recv_handler(&self, on_recv: dl::Handler) {
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
