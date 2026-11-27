use std::time::Duration;

use crate::clock::game_clock::GameClock;
use crate::feed::loop_frame::LoopFrame;
use crate::loop_stepper::LoopStepper;
use crate::tests::test_state::{TestCommand, TestFrame, TestState};

fn stepper() -> LoopStepper<TestState> {
  LoopStepper::new(Duration::from_millis(10), GameClock::default(), TestState::default())
}

#[test]
fn moves_the_game_day_by_its_factor_each_tick() {
  let mut stepper: LoopStepper<TestState> = stepper();
  let start: f32 = stepper.get_frame().time_of_day;

  for _ in 0..100 {
    stepper.step([]);
  }

  assert_eq!(stepper.get_time().tick, 100);
  // A simulated second at ten game seconds a second.
  assert!((stepper.get_frame().time_of_day - start - 10.0).abs() < 1e-3);
}

#[test]
fn applies_the_commands_in_order_before_the_step_from_its_tick() {
  let mut stepper: LoopStepper<TestState> = stepper();

  stepper.step([TestCommand::SetSpeed(2.0), TestCommand::Add(1.0)]);
  stepper.step([TestCommand::SetSpeed(0.5)]);

  // The system moved by the speed its root handed it each step.
  assert_eq!(stepper.get_state().counter.value, 3.5);
  // A step reads the tick it steps from.
  assert_eq!(stepper.get_state().counter.ticks, vec![0, 1]);
  assert_eq!(stepper.get_frame().tick, 2);
  assert_eq!(stepper.get_frame().state, TestFrame(3.5));
}

#[test]
fn steps_a_recorded_session_to_the_same_frames_every_time() {
  let session: Vec<Vec<TestCommand>> = vec![
    vec![TestCommand::SetSpeed(0.25)],
    vec![],
    vec![TestCommand::Add(3.0), TestCommand::SetSpeed(-1.0)],
    vec![TestCommand::Add(0.5)],
  ];
  let replay = || -> Vec<LoopFrame<TestFrame>> {
    let mut stepper: LoopStepper<TestState> = stepper();

    session
      .iter()
      .map(|commands| {
        stepper.step(commands.iter().copied());
        stepper.get_frame()
      })
      .collect()
  };

  assert_eq!(replay(), replay());
}
