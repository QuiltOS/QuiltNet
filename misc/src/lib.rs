#![feature(unboxed_closures, fn_traits)]

extern crate core;

#[macro_use]
extern crate log;

pub mod interface;
pub mod state_machine;

use core::ops::FnMut;

use std::sync::mpsc::Sender;
use std::sync::Mutex;

pub struct LockedClosure<F> {
  pub closure: Mutex<F>
}

impl<F, Args> FnOnce<Args> for LockedClosure<F> where F: FnMut<Args> + Send {
  type Output = <F as FnOnce<Args>>::Output;

  extern "rust-call" fn call_once(mut self, args: Args) -> Self::Output {
    self.call_mut(args)
  }

}

impl<F, Args> FnMut<Args> for LockedClosure<F> where F: FnMut<Args> + Send {
  extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output {
    self.closure.get_mut().unwrap().call_mut(args)
  }
}


impl<F, Args> Fn<Args> for LockedClosure<F> where F: FnMut<Args> + Send {
  extern "rust-call" fn call(&self, args: Args) -> Self::Output {
    self.closure.lock().unwrap().call_mut(args)
  }
}

// TODO get rid of mutex
pub struct SenderClosure<T> {
  pub sender: Mutex<Sender<T>>
}

impl<T> SenderClosure<T> {
  pub fn new(sender: Sender<T>) -> SenderClosure<T> {
    SenderClosure { sender: Mutex::new(sender) }
  }
}

impl<T> FnOnce<T> for SenderClosure<T> {
  type Output = ();

  extern "rust-call" fn call_once(mut self, args: T) -> () {
    self.call_mut(args);
  }
}

impl<T> FnMut<T> for SenderClosure<T> {
  extern "rust-call" fn call_mut(&mut self, args: T) -> () {
    debug!("SenderClosure called without lock!");
    self.sender.get_mut().unwrap().send(args).unwrap();
  }
}

impl<T> Fn<T> for SenderClosure<T> {
  extern "rust-call" fn call(&self, args: T) -> () {
    debug!("SenderClosure called with lock!");
    self.sender.lock().unwrap().send(args).unwrap();
  }
}
