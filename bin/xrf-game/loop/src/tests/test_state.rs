use crate::clock::loop_time::LoopTime;
use crate::state::loop_blend::LoopBlend;
use crate::state::loop_state::LoopState;
use crate::state::loop_system::LoopSystem;

/// A composition root holding a speed and one system, which it hands the speed each step.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TestState {
  pub speed: f32,
  pub counter: TestCounter,
}

/// A system that adds the numbers it is sent and moves its value on by the speed it is handed, recording the ticks it
/// saw.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct TestCounter {
  pub value: f32,
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

impl LoopSystem for TestCounter {
  type Command = f32;
  type Context<'a> = &'a f32;

  fn apply(&mut self, value: f32) {
    self.value += value;
  }

  fn step(&mut self, time: &LoopTime, speed: &f32) {
    self.value += speed;
    self.ticks.push(time.tick);
  }
}

impl LoopState for TestState {
  type Command = TestCommand;
  type Frame = TestFrame;

  fn apply(&mut self, command: TestCommand) {
    match command {
      TestCommand::Add(value) => self.counter.apply(value),
      TestCommand::SetSpeed(speed) => self.speed = speed,
    }
  }

  fn step(&mut self, time: &LoopTime) {
    self.counter.step(time, &self.speed);
  }

  fn get_frame(&self) -> TestFrame {
    TestFrame(self.counter.value)
  }
}
