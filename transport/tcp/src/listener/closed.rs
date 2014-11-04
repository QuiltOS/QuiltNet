use super::Listener;
use super::state::State;
use super::state::InputEvent;

pub struct Closed;

impl State for Closed
{
  fn next(self, _i: InputEvent) -> Listener {
    super::Listen(super::listen::Listen)
  }
}
