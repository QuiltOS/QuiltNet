use std::collections::hashmap::HashMap;

use std::io::IoResult;
use std::io::net;
use std::io::net::udp::UdpSocket;
use std::io::net::ip::{SocketAddr, Ipv4Addr, Port};

use std::sync::Arc;
use std::sync::RWLock;

use std::task::spawn;

use interface::{Handler, Interface};
use super::DataLinkInterface;

static RECV_BUF_SIZE: uint = 64 * 1024;

// Closure has mutable environment, therefore need to lock each one
type SharedHandlerMap = Arc<RWLock<HashMap<SocketAddr, Handler>>>;

/// The backing listening socket / read loop for a bunch of UDP-backed mock link interfaces
pub struct Listener {
    socket:   UdpSocket,
    handlers: SharedHandlerMap,
}

impl Listener {

    pub fn new(listen_port: net::ip::Port) -> IoResult<Listener>
    {
        let socket = try!(UdpSocket::bind(
            SocketAddr { ip:  Ipv4Addr(0, 0, 0, 0), port: listen_port }));

        let handlers: SharedHandlerMap = Arc::new(RWLock::new(HashMap::new()));

        spawn({
            let mut socket   = socket.clone();
            let     handlers = handlers.clone();
            proc() {
                // TODO: shouldn't need to initialize this
                let mut buf: [u8, ..RECV_BUF_SIZE] = [0, ..RECV_BUF_SIZE];
                loop {
                    // select! (
                    //     () = die.recv() => {
                    //         println!("I am going to die");
                    //         return;
                    //     },
                    match socket.recv_from(buf.as_mut_slice()) {
                        Err(_) => {
                            // maybe it will work next time...
                            println!("something bad happened");
                        },
                        Ok((len, src_addr)) => match handlers.read().deref().find(&src_addr) {
                            None          => continue, // drop that packet!
                            Some(on_recv) => {
                                let args = (buf[..len].to_vec(),);
                                (&**on_recv).call(args);
                            },
                        }
                    };
                    // )
                }
            }
        });

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
pub struct UdpMockDataLinkInterface {
    listener:    Listener,
    remote_addr: SocketAddr,
}


impl UdpMockDataLinkInterface {

    pub fn new(listener:    &Listener,
               remote_addr: SocketAddr,
               on_recv:     Handler) -> UdpMockDataLinkInterface
    {
        listener.handlers.write().deref_mut().insert(remote_addr, on_recv);

        UdpMockDataLinkInterface {
            listener:    listener.clone(),
            remote_addr: remote_addr,
        }
    }
}

impl Interface for UdpMockDataLinkInterface {
    fn send(&mut self, packet: Box<[u8]>) -> IoResult<()> {
        try!(self.listener.socket.send_to(packet.as_slice(), self.remote_addr));
        Ok(())
    }

    fn update_recv_handler(&mut self, on_recv: Handler) {
        self.listener
            .handlers
            .write()
            .deref_mut()
            .insert(self.remote_addr,
                    on_recv);
    }
}

impl DataLinkInterface for UdpMockDataLinkInterface {

    fn enable(&mut self) {

    }

    fn disable(&mut self) {

    }

}

#[test]
fn it_works() {
}
