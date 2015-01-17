#![allow(unstable)]

#![feature(box_syntax)]
#![feature(unboxed_closures)]
#![feature(slicing_syntax)]


#[macro_use] #[no_link]
extern crate log_ng;

//#[cfg(any(log_level = "error",
//          log_level = "warn",
//          log_level = "info",
//          log_level = "debug",
//          log_level = "trace"))]
extern crate log_ng;

#[macro_use]
extern crate misc;
extern crate "interface" as dl;

use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::io::IoResult;
use std::io::net::udp::UdpSocket;
use std::io::net::ip::{SocketAddr};

use std::sync::Arc;
use std::sync::RwLock;

use std::thread::Thread;

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
  pub fn new(listen_addr: SocketAddr, num_threads: usize) -> IoResult<Listener>
  {
    assert!(num_threads > 0);

    let socket = try!(UdpSocket::bind(listen_addr));

    let handlers: SharedHandlerMap = Arc::new(RwLock::new(HashMap::new()));


    for _ in range(0, num_threads) {
      let mut socket   = socket.clone();
      let     handlers = handlers.clone();
      Thread::spawn(move || {
        let mut buf: [u8; RECV_BUF_SIZE] = unsafe { std::mem::uninitialized() };
        loop {
          // TODO: someway to kill task eventually
          match socket.recv_from(buf.as_mut_slice()) {
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
                  (**on_recv).call((args,));
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
}

impl Clone for Listener {

  fn clone(&self) -> Listener {
    Listener {
      socket:   self.socket  .clone(),
      handlers: self.handlers.clone(),
    }
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
      listener:      listener.clone(),
      remote_addr:   remote_addr,
      cached_status: true,
    }
  }
}

impl root::Interface for Interface {

}

impl dl::Interface for Interface
{
  fn send(&self, packet: dl::Packet) -> dl::Result<()> {

    if self.cached_status == false {
      return Err(dl::Error::Disabled);
    }

    let mut socket = self.listener.socket.clone();
    socket.send_to(packet.as_slice(), self.remote_addr).map_err(dl::Error::External)
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
