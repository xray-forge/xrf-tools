use std::time::Duration;

use crate::feed::loop_frame::LoopFrame;
use crate::loop_stepper::LoopStepper;
use crate::tests::test_state::{TestCommand, TestFrame, TestState};

fn stepper() -> LoopStepper<TestState> {
  LoopStepper::new(Duration::from_millis(10), TestState::default())
}

#[test]
fn counts_its_ticks_by_its_step() {
  let mut stepper: LoopStepper<TestState> = stepper();

  for _ in 0..100 {
    stepper.step([]).expect("a step");
  }

  assert_eq!(stepper.get_time().tick, 100);
  assert_eq!(stepper.get_time().step, Duration::from_millis(10));
}

#[test]
fn applies_the_commands_in_order_before_the_step_from_its_tick() {
  let mut stepper: LoopStepper<TestState> = stepper();

  let told: Vec<f32> = stepper
    .step([TestCommand::SetSpeed(2.0), TestCommand::Add(1.0)])
    .expect("a step");

  stepper.step([TestCommand::SetSpeed(0.5)]).expect("a step");

  // The system moved by the speed its root handed it each step, and the step told what it left.
  assert_eq!(told, vec![3.0]);
  assert_eq!(stepper.get_state().counter.value, 3.5);
  // A step reads the tick it steps from.
  assert_eq!(stepper.get_state().counter.ticks, vec![0, 1]);
  assert_eq!(stepper.get_frame().tick, 2);
  assert_eq!(stepper.get_frame().state, TestFrame(3.5));
}

#[test]
fn answers_a_fault_and_holds_its_tick() {
  let mut stepper: LoopStepper<TestState> = stepper();

  stepper.step([]).expect("a step");

  assert!(stepper.step([TestCommand::Fail]).is_err());
  assert_eq!(stepper.get_time().tick, 1);
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
        stepper.step(commands.iter().copied()).expect("a step");
        stepper.get_frame()
      })
      .collect()
  };

  assert_eq!(replay(), replay());
}
