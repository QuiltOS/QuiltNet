use std::io::IoResult;
use std::io::net::ip::{SocketAddr, IpAddr, Ipv4Addr, Port};
use std::io::net::udp::UdpSocket;

use std::sync::{Arc, Barrier};
use std::string::String;


use data_link::{DLHandler, DLInterface};

use super::*;

// can't use this until #17543 lands
//fn simple_wait(packet: Vec<u8>) {
//    assert_eq!(packet[0], 0u8);
//}

#[test]
fn echo() {
    println!("hello tester!");
}

fn mk_listener() -> IoResult<(Listener, SocketAddr)> {
    // port 0 is dynamically assign
    let mut listener = try!(Listener::new(0));
    let mut addr     = try!(listener.socket.socket_name());
    addr.ip = Ipv4Addr(127, 0, 0, 1);
    println!("made listern with addr: {}", addr);
    Ok((listener, addr))
}

#[test]
fn talk_to_self() {
    fn inner() -> IoResult<()> {
        let barrier = Arc::new(Barrier::new(3));

        let (l1, a1) = try!(mk_listener());
        let (l2, a2) = try!(mk_listener());

        let mk_callback = | msg: String | -> DLHandler {
            let barrier = barrier.clone();
            let msg     = msg.into_bytes();
            box |&: packet: Vec<u8> | {
                println!("got packet: {}", packet);
                assert_eq!(packet, msg);
                barrier.wait();
            }
        };

        // avoid ICE
        struct TempClosure {
            expect:  u8,
            barrier: Arc<Barrier>,
        }

        impl Fn<(Vec<u8>,), ()> for TempClosure {
            #[rust_call_abi_hack]
            fn call(&self, packet: (Vec<u8>,)) {

            }
        }

        static M1: &'static str = "Hey Josh!";
        static M2: &'static str = "Hey Cody!";

        let interface1 = UdpMockDLInterface::new(&l1, a2, mk_callback(String::from_str(M1)));
        let interface2 = UdpMockDLInterface::new(&l2, a1, mk_callback(String::from_str(M2)));

        try!(interface1.send(String::from_str(M1).into_bytes()));
        try!(interface2.send(String::from_str(M1).into_bytes()));

        barrier.wait();

        Ok(())
    }

    inner().ok().unwrap();

}
