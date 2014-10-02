use std::io::IoResult;

pub mod udp_mock_link;

// TODO: use Box<[u8]> instead of Vec<u8>
pub type Handler = fn (&[Vec<u8>]) -> ();

pub trait Interface {

    /// Send data-link-layer packet with specified body
    fn send(&mut self, packet: Box<[u8]>) -> IoResult<()>;

    /// Update the function called on an arriving packet
    fn update_recv_handler(&mut self, on_recv: Handler);

    //type Err;

    //fn new(on_receive: |Vec<u8>| -> ()) -> Self;

    //fn stock(&mut Self, bufs: Box<[Box<[u8]>]>);

    //fn kill(&Self);
}
