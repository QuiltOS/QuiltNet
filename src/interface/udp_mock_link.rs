use std::collections::hashmap::HashMap;

use std::io::IoResult;
use std::io::net;
use std::io::net::udp::UdpSocket;
use std::io::net::ip::{SocketAddr, Ipv4Addr, Port};

use std::sync::Arc;
use std::sync::RWLock;

use std::task::spawn;



static RECV_BUF_SIZE: uint = 2*2*2*2*2*2*2*2;

/// The backing listening socket / read loop for a bunch of UDP-backed mock link interfaces
pub struct Listener {
    socket:   UdpSocket,
    handlers: Arc<RWLock<HashMap<SocketAddr, super::Handler>>>,
}

impl Listener {

    pub fn new(listen_port: net::ip::Port) -> IoResult<Listener>
    {
        let socket = try!(UdpSocket::bind(
            SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: listen_port }));

        let handlers: Arc<RWLock<HashMap<SocketAddr, super::Handler>>>
            = Arc::new(RWLock::new(HashMap::new()));

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
                            println!("something bad happend");
                        },
                        Ok((len, src_addr)) => match handlers.read().deref().find(&src_addr) {
                            None          => continue, // drop that packet!
                            Some(on_recv) => {
                                // real network card may consolidate multiple packets per interrupt
                                let bufs: [Vec<u8>, ..1] = [buf.slice_to(len).to_vec()];
                                (*on_recv)(bufs.as_slice());
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
pub struct LinkInterface {
    listener:    Listener,
    remote_addr: SocketAddr,
}


impl LinkInterface {

    pub fn new(listener:    &Listener,
               remote_addr: SocketAddr,
               on_recv:     super::Handler) -> LinkInterface
    {
        listener.handlers.write().deref_mut().insert(remote_addr, on_recv);

        LinkInterface {
            listener:    listener.clone(),
            remote_addr: remote_addr,
        }
    }

    pub fn update_recv(&self, on_recv: super::Handler) {
        self.listener
            .handlers
            .write()
            .deref_mut()
            .insert(self.remote_addr,
                    on_recv);
    }

    pub fn send(&mut self, buf: &[u8]) -> IoResult<()> {
        try!(self.listener.socket.send_to(buf, self.remote_addr));
        Ok(())
    }
}
