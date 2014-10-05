use std::io::IoResult;

use data_link::DLInterface;

use super::*;

// can't use this until #17543 lands
//fn simple_wait(packet: Vec<u8>) {
//    assert_eq!(packet[0], 0u8);
//}

#[test]
fn talk_to_self() {
    fn inner() -> IoResult<()> {
        // port 0 is dynamically assign
        let mut listener1 = try!(Listener::new(0));
        let addr1         = try!(listener1.socket.socket_name());

        let mut listener2 = try!(Listener::new(0));
        let addr2         = try!(listener2.socket.socket_name());

        // avoid ICE
        struct TempNonClosure;

        impl Fn<(Vec<u8>,), ()> for TempNonClosure {
            #[rust_call_abi_hack]
            fn call(&self, packet: (Vec<u8>,)) {
                match packet {
                    (packet,) => {
                        println!("got packet {}", packet);
                        assert_eq!(packet[0], 0u8);
                    }
                }
            }
        }

        let mut interface1 = UdpMockDLInterface::new(&listener1, addr2, box TempNonClosure);
        let mut interface2 = UdpMockDLInterface::new(&listener2, addr1, box TempNonClosure);

        try!(interface1.send(vec!(0)));
        try!(interface2.send(vec!(0)));
        Ok(())
    }

    inner().ok().unwrap();

}
