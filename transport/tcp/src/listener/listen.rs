use super::Listener;
use super::state::State;
use super::state::InputEvent;

pub struct Listen;

impl State for Listen
{
  fn next(self, _i: InputEvent) -> Listener {
    super::Closed(super::closed::Closed)
  }
}
