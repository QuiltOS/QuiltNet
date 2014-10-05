use std::io::IoResult;
use std::io::net::ip::{SocketAddr, Ipv4Addr, Port};

use std::sync::{Arc, Barrier};


use data_link::DLInterface;

use super::*;

// can't use this until #17543 lands
//fn simple_wait(packet: Vec<u8>) {
//    assert_eq!(packet[0], 0u8);
//}

#[test]
fn echo() {
    println!("hello tester!");
}

#[test]
fn talk_to_self() {
    fn inner() -> IoResult<()> {
        let barrier = Arc::new(Barrier::new(3));

        // port 0 is dynamically assign
        let mut listener1 = try!(Listener::new(0));
        let addr1         = {
            let mut addr = try!(listener1.socket.socket_name());
            addr.ip = Ipv4Addr(127, 0, 0, 1);
            addr
        };
        println!("addr1: {}", addr1);

        let mut listener2 = try!(Listener::new(0));
        let addr2         = {
            let mut addr = try!(listener1.socket.socket_name());
            addr.ip = Ipv4Addr(127, 0, 0, 1);
            addr
        };
        println!("addr2: {}", addr2);

        // avoid ICE
        struct TempClosure {
            expect:  u8,
            barrier: Arc<Barrier>,
        }

        impl Fn<(Vec<u8>,), ()> for TempClosure {
            #[rust_call_abi_hack]
            fn call(&self, packet: (Vec<u8>,)) {
                match packet {
                    (packet,) => {
                        println!("got packet: {}", packet);
                        assert_eq!(packet[0], self.expect);
                        self.barrier.wait();
                    }
                }
            }
        }

        let mut interface1 = UdpMockDLInterface::new(
            &listener1, addr2, box TempClosure{ expect: 1, barrier: barrier.clone() });
        let mut interface2 = UdpMockDLInterface::new(
            &listener2, addr1, box TempClosure{ expect: 0, barrier: barrier.clone() });

        try!(interface1.send(vec!(0)));
        try!(interface2.send(vec!(1)));

        barrier.wait();

        Ok(())
    }

    inner().ok().unwrap();

}
