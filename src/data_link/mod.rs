use std::io::IoResult;

pub use interface::{Interface, Handler};

pub mod udp_mock;

// TODO: use Box<[u8]> instead of Vec<u8>
pub type DLPacket = Vec<u8>;

pub type DLHandler = Handler<DLPacket>;

pub trait DLInterface: Interface {

    /// Send packet with specified body
    fn send(&self, packet: DLPacket) -> IoResult<()>;

    /// Update the function called on an arriving packet
    fn update_recv_handler(&self, on_recv: DLHandler);

    //type Err;

    //fn new(on_receive: |Vec<u8>| -> ()) -> Self;

    //fn stock(&mut Self, bufs: Box<[Vec<u8>]>);

    //fn kill(&Self);

    fn enable(&self);
    fn disable(&self);
}
