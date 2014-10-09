//use core::ops::Fn;
//use core::ops::FnMut;

use std::comm::Sender;
use std::sync::Mutex;
use std::io::IoResult;


// TODO: real network card may consolidate multiple packets per interrupt.
pub type Handler<Packet> = Box<Fn<(Packet,), ()> + Send + 'static>;
//pub type Handler<Packet> = <|&: Packet|:Send -> ()>;

pub trait Interface {
    // need associated types to be better
}

// might not work, see rust-lang #17779

pub struct LockedClosure<F> {
    pub closure: Mutex<F>
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
    pub sender: Sender<T>
}

impl<T> SenderClosure<T> {
    pub fn new(sender: Sender<T>) -> SenderClosure<T> {
        SenderClosure { sender: sender }
    }
}

impl<T> Fn<T, ()> for SenderClosure<T>
    where T: Send
{
    #[rust_call_abi_hack]
    fn call(&self, args: T) -> () {
        self.sender.send(args);
    }
}
