use super::{
  Listener,

  Closed,
  Listen,
};

pub struct InputEvent;

pub trait State {
  fn next(self, InputEvent) -> Listener;
}

pub fn trans<S>(e: &mut Listener, i: InputEvent) {
  *e = match e {
    &Closed(ref s) => s.next(i),
    &Listen(ref s) => s.next(i),
  }
}
