//use core::ops::Fn;
//use core::ops::FnMut;

use std::comm::Sender;
use std::sync::Mutex;
use std::io::IoResult;

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

// TODO: real network card may consolidate multiple packets per interrupt.
pub type Handler<Packet> = Box<Fn<(Packet,), ()> + Send + 'static>;
//pub type Handler<Packet> = <|&: Packet|:Send -> ()>;

pub trait Interface {
    // need associated types to be better
}

// might not work, see rust-lang #17779

pub struct LockedClosure<F> {
    closure: Mutex<F>
}

impl<F, Args, Result> Fn<Args, Result> for LockedClosure<F>
    where F: FnMut<Args, Result>, F:Send
{
    #[rust_call_abi_hack]
    fn call(&self, args: Args) -> Result {
        self.closure.lock().deref_mut().call_mut(args)
    }
}

pub struct SenderClosure<T> {
    sender: Sender<T>
}

impl<T> Fn<T, ()> for SenderClosure<T>
    where T: Send
{
    #[rust_call_abi_hack]
    fn call(&self, args: T) -> () {
        self.sender.send(args);
    }
}
