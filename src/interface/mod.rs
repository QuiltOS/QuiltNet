use std::io::IoResult;

pub mod udp_mock_link;

// TODO: use Box<[u8]> instead of Vec<u8>
pub type Handler = fn (&[Vec<u8>]) -> ();

pub trait Interface {

    //type Err;

    // TODO: use Box<[u8]> instead of Vec<u8>

    //fn new(on_receive: |Vec<u8>| -> ()) -> Self;

    /// Send data-link-layer packet with specified body
    fn send(&mut Self, body: Box<[u8]>) -> IoResult<()>;

    //fn stock(&mut Self, bufs: Box<[Box<[u8]>]>);

    fn kill(&Self);
}
