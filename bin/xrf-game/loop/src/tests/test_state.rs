use crate::clock::loop_time::LoopTime;
use crate::state::loop_blend::LoopBlend;
use crate::state::loop_state::LoopState;
use crate::state::loop_system::LoopSystem;

/// A state that adds the numbers it is sent and moves its value on by its speed each step, recording the ticks it saw.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TestState {
  pub value: f32,
  pub speed: f32,
  pub ticks: Vec<u64>,
}

/// A command to the test state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum TestCommand {
  Add(f32),
  SetSpeed(f32),
}

/// The test state's value as published.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TestFrame(pub f32);

impl LoopBlend for TestFrame {
  fn blend(&self, next: &Self, alpha: f32) -> Self {
    Self(self.0 + (next.0 - self.0) * alpha)
  }
}

impl LoopSystem for TestState {
  type Command = TestCommand;

  fn apply(&mut self, command: TestCommand) {
    match command {
      TestCommand::Add(value) => self.value += value,
      TestCommand::SetSpeed(speed) => self.speed = speed,
    }
  }

  fn step(&mut self, time: &LoopTime) {
    self.value += self.speed;
    self.ticks.push(time.tick);
  }
}

impl LoopState for TestState {
  type Frame = TestFrame;

  fn get_frame(&self) -> TestFrame {
    TestFrame(self.value)
  }
}
