use std::io::IoResult;
use std::io::net::ip::{SocketAddr, IpAddr, Ipv4Addr, Port};
use std::io::net::udp::UdpSocket;

use std::sync::{Arc, Barrier};
use std::string::String;

use interface::SenderClosure;

use data_link::{DLPacket, DLHandler, DLInterface};

use super::*;

fn mk_listener(num_threads: uint) -> IoResult<(Listener, SocketAddr)> {
    // port 0 is dynamically assign
    let mut listener = try!(Listener::new(0, num_threads));
    let mut addr     = try!(listener.socket.socket_name());
    addr.ip = Ipv4Addr(127, 0, 0, 1);
    println!("made listener with addr: {}", addr);
    Ok((listener, addr))
}

fn talk_to_self_channel_helper(num_threads: uint) {
    use std::comm;

    fn inner(num_threads: uint) -> IoResult<()> {
        let (l1, a1) = try!(mk_listener(num_threads));
        let (l2, a2) = try!(mk_listener(num_threads));

        let (tx1, rx1) = channel::<(DLPacket,)>();
        let (tx2, rx2) = channel::<(DLPacket,)>();

        static M1: &'static str = "Hey Josh!";
        static M2: &'static str = "Hey Cody!";

        let interface1 = UdpMockDLInterface::new(&l1, a2, box SenderClosure { sender: tx1 });
        let interface2 = UdpMockDLInterface::new(&l2, a1, box SenderClosure { sender: tx2 });

        try!(interface1.send(String::from_str(M2).into_bytes()));
        try!(interface2.send(String::from_str(M1).into_bytes()));

        let (packet_1,) = rx1.recv();
        assert_eq!(packet_1.as_slice(), M1.as_bytes());
        println!("Got the first packet");

        let (packet_2,) = rx2.recv();
        assert_eq!(packet_2.as_slice(), M2.as_bytes());
        println!("Got the second packet");

        Ok(())
    }

    inner(num_threads).unwrap();

}

#[test]
fn talk_to_self_channel() {
    talk_to_self_channel_helper(1);
}
#[test]
fn talk_to_self_channel_parallel() {
    talk_to_self_channel_helper(4);
}


fn talk_to_self_callback_helper(num_threads: uint) {
    fn inner(num_threads: uint) -> IoResult<()> {
        let barrier = Arc::new(Barrier::new(3));

        let (l1, a1) = try!(mk_listener(num_threads));
        let (l2, a2) = try!(mk_listener(num_threads));

        let mk_callback = | msg: String | -> DLHandler {
            let barrier = barrier.clone();
            let msg     = msg.into_bytes();
            box |&: packet: Vec<u8> | -> () {
                println!("got packet: {}", packet);
                assert_eq!(packet, msg);
                barrier.wait();
            }
        };

        static M1: &'static str = "Hey Josh!";
        static M2: &'static str = "Hey Cody!";

        let interface1 = UdpMockDLInterface::new(&l1, a2, mk_callback(String::from_str(M1)));
        let interface2 = UdpMockDLInterface::new(&l2, a1, mk_callback(String::from_str(M2)));

        try!(interface1.send(String::from_str(M2).into_bytes()));
        try!(interface2.send(String::from_str(M1).into_bytes()));

        barrier.wait();

        Ok(())
    }

    inner(num_threads).unwrap();

}
#[ignore]
#[test]
fn talk_to_self_callback() {
    talk_to_self_callback_helper(1);
}
#[ignore]
#[test]
fn talk_to_self_callback_parallel() {
    talk_to_self_callback_helper(4);
}
