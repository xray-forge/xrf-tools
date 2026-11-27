use std::time::{Duration, Instant};

use crate::clock::game_clock::GameClock;
use crate::clock::loop_clock::LoopClock;
use crate::loop_stepper::LoopStepper;
use crate::tests::test_state::{TestCommand, TestFrame, TestState};
use crate::thread::game_loop::GameLoop;

#[test]
fn steps_on_its_thread_and_takes_commands_until_dropped() {
  let clock: LoopClock = LoopClock::new(Instant::now(), LoopClock::DEFAULT_TICK_RATE);
  let stepper: LoopStepper<TestState> = LoopStepper::new(clock.get_step(), GameClock::default(), TestState::default());
  let thread: GameLoop<TestState> = GameLoop::start(stepper, clock).expect("the thread starts");

  assert_eq!(thread.get_feed().get_latest().map(|frame| frame.tick), Some(0));

  thread
    .get_sender()
    .send(TestCommand::Add(7.0))
    .expect("the thread takes commands");

  let deadline: Instant = Instant::now() + Duration::from_secs(5);

  while thread.get_feed().get_latest().map(|frame| frame.state) != Some(TestFrame(7.0)) {
    assert!(Instant::now() < deadline, "the command never reached a published frame");
    std::thread::sleep(Duration::from_millis(1));
  }

  let sender = thread.get_sender();

  drop(thread);

  // Stopped, it takes no more.
  assert!(sender.send(TestCommand::Add(1.0)).is_err());
}
