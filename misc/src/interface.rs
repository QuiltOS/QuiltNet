use std::comm::Sender;
use std::sync::Mutex;
// TODO: get rid of `pub` in `pub use`
pub use std::ops::Fn;


pub struct Nop;

impl<T> Fn<T, ()> for Nop where T: Send
{
  extern "rust-call" fn call(&self, _: T) { }
}



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
  extern "rust-call" fn call(&self, args: Args) -> Result {
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

impl<T> Fn<T, ()> for SenderClosure<T> where T: Send
{
  extern "rust-call" fn call(&self, args: T) -> () {
    debug!("SenderClosure called!");
    self.sender.send(args);
  }
}
