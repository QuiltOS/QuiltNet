pub enum Next<State, Input> {
  Terminate,
  Continue(Transitions<State, Input>),
}

pub type Transitions<State, Input> = for<'a> fn(&'a mut State, Input) -> Next<State, Input>;

pub struct StateMachine<State, Input> {
  state:       State,
  transitions: Transitions<State, Input>
}

impl<S, I> StateMachine<S, I> {

  pub fn new(initial_state:       S,
             initial_transitions: Transitions<S, I>)
             -> StateMachine<S, I>
  {
    StateMachine {
      state:       initial_state,
      transitions: initial_transitions,
    }
  }

  // returns false if terminating
  pub fn next(&mut self, input: I) -> bool {
    match (self.transitions)(&mut self.state, input) {
      Next::Terminate        => false,
      Next::Continue(new_ts) => {
        self.transitions = new_ts;
        true
      },
    }
  }

}
