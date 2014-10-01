use std::io::IoResult;
use std::io::net;
use std::io::net::udp::UdpSocket;
use std::io::net::ip::{SocketAddr, Ipv4Addr, Port};

use std::task::spawn;

// TODO: use Box<[u8]> instead of Vec<u8>

/// A mock link layer interface made from UDP, good for testing
pub struct UdpLink {
    pub socket: UdpSocket,
    //pub kill:   Sender<()>,
}

pub fn new(src_port:   net::ip::Port,
           dst:        net::ip::SocketAddr,
           on_receive: fn (&[Vec<u8>]) -> ()) -> IoResult<UdpLink>
{
    let socket = try!(UdpSocket::bind(
            SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: src_port }));

    //let (kill, die) = channel();

    spawn({
        let mut socket = socket.clone().connect(dst);
        proc() {
            socket = socket;
            loop {
                // select! (
                //     () = die.recv() => {
                //         println!("I am going to die");
                //         return;
                //     },
                match socket.read_to_end() {
                    Err(_) => {
                        // maybe it will work next time...
                        println!("something bad happend");
                    },
                    Ok(buf) => {
                        let bufs: [Vec<u8>, ..1] = [buf];
                        println!("stuff arrived");
                        on_receive(bufs.as_slice());
                    },
                }
                // )
            }
        }
    });

    Ok(UdpLink {
        socket: socket,
        //kill: kill,
    })
}
