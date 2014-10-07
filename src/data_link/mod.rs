use std::io::IoResult;

use interface::{Handler, Interface};

pub mod udp_mock;

// TODO: use Box<[u8]> instead of Vec<u8>
pub type DLPacket = Box<[u8]>;

pub type DLHandler = Handler<DLPacket>;

pub trait DLInterface: Interface {

    /// Send packet with specified body
    fn send(&mut self, packet: DLPacket) -> IoResult<()>;

    /// Update the function called on an arriving packet
    fn update_recv_handler(&mut self, on_recv: DLHandler);

    //type Err;

    //fn new(on_receive: |Vec<u8>| -> ()) -> Self;

    //fn stock(&mut Self, bufs: Box<[Box<[u8]>]>);

    //fn kill(&Self);

    fn enable(&mut self);
    fn disable(&mut self);
}
