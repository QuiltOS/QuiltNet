
use std::sync::mpsc::Sender;
use std::sync::Mutex;


// TODO: real network card may consolidate multiple packets per interrupt.
pub type Handler<Packet> = Box<Fn<(Packet,), ()> + Send + 'static>;
//pub type Handler<Packet> = <|&: Packet|:Send -> ()>;

pub trait Interface {
  // need associated types to be better
}

pub struct LockedClosure<F> {
  pub closure: Mutex<F>
}

impl<F, Args, Res> Fn<Args, Res> for LockedClosure<F>
  where F: FnMut<Args, Res>, F:Send
{
  extern "rust-call" fn call(&self, args: Args) -> Res {
    self.closure.lock().unwrap().call_mut(args)
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
    self.sender.send(args).unwrap();
  }
}
