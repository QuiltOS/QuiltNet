//use core::ops::Fn;
//use core::ops::FnMut;

//use std::comm::Sender;
use std::io::IoResult;
//use std::sync::Mutex;



// TODO: use Box<[u8]> instead of Vec<u8>
// TODO: real network card may consolidate multiple packets per interrupt.
pub type Handler = Box<Fn<(Vec<u8>), ()> + Send + 'static>;

pub trait Interface {

    /// Send packet with specified body
    fn send(&mut self, packet: Box<[u8]>) -> IoResult<()>;

    /// Update the function called on an arriving packet
    fn update_recv_handler(&mut self, on_recv: Handler);
}

// won't work, see rust-lang #17779
//
//pub struct LockedClosure<F> {
//    closure: Mutex<F>
//}
//
//impl<F, Args, Result> Fn<Args, Result> for LockedClosure<F>
//    where F: FnMut<Args, Result>, F:Send
//{
//    #[rust_call_abi_hack]
//    fn call(&self, args: Args) -> Result {
//        self.closure.lock().deref_mut().call_mut(args)
//    }
//}

//pub struct SenderClosure<T> {
//    sender: Sender<T>
//}
//
//impl<T> Fn<(T, ()), ()> for SenderClosure<T>
//    where T: Send
//{
//    #[rust_call_abi_hack]
//    fn call(&self, args: (T, ())) -> () {
//        match args {
//            (arg, _) => self.sender.send(arg)
//        }
//    }
//}
