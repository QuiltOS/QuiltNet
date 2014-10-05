//use core::ops::Fn;
//use core::ops::FnMut;

use std::comm::Sender;
use std::sync::Mutex;



// TODO: real network card may consolidate multiple packets per interrupt.
pub type Handler<Packet> = Box<Fn<(Packet,), ()> + Send + 'static>;

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