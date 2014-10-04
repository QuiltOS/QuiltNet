use interface::Interface;

pub mod udp_mock;

pub trait DataLinkInterface: Interface {

    //type Err;

    //fn new(on_receive: |Vec<u8>| -> ()) -> Self;

    //fn stock(&mut Self, bufs: Box<[Box<[u8]>]>);

    //fn kill(&Self);

    fn enable(&mut self);
    fn disable(&mut self);
}
