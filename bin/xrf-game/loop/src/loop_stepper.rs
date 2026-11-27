use std::time::Duration;

use crate::clock::loop_time::LoopTime;
use crate::feed::loop_frame::LoopFrame;
use crate::state::loop_state::LoopState;

/// The game's state and the fixed step that moves it: a pure function of its state and the commands a tick takes,
/// with no clock of its own, so a test steps it as the thread does.
#[derive(Clone, Debug)]
pub struct LoopStepper<S> {
  time: LoopTime,
  state: S,
}

impl<S: LoopState> LoopStepper<S> {
  /// `state` at tick 0, stepping by `step`.
  pub fn new(step: Duration, state: S) -> Self {
    Self {
      time: LoopTime::new(step),
      state,
    }
  }

  pub fn get_time(&self) -> &LoopTime {
    &self.time
  }

  pub fn get_state(&self) -> &S {
    &self.state
  }

  /// Takes the commands that arrived since the last tick, in the order they arrived, and moves the state on a step.
  pub fn step(&mut self, commands: impl IntoIterator<Item = S::Command>) {
    for command in commands {
      self.state.apply(command);
    }

    self.state.step(&self.time);
    self.time = self.time.next();
  }

  /// What presentation reads of the state as it stands.
  pub fn get_frame(&self) -> LoopFrame<S::Frame> {
    LoopFrame {
      tick: self.time.tick,
      state: self.state.get_frame(),
    }
  }
}
