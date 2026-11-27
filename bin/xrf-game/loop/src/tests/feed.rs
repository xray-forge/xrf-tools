use std::time::{Duration, Instant};

use crate::feed::loop_feed::LoopFeed;
use crate::feed::loop_frame::LoopFrame;
use crate::tests::test_state::TestFrame;

fn frame(tick: u64, value: f32, time_of_day: f32) -> LoopFrame<TestFrame> {
  LoopFrame {
    tick,
    time_of_day,
    time_factor: 10.0,
    state: TestFrame(value),
  }
}

#[test]
fn reads_nothing_before_the_first_frame_and_that_frame_alone_after_it() {
  let feed: LoopFeed<TestFrame> = LoopFeed::default();
  let now: Instant = Instant::now();

  assert_eq!(feed.read(now), None);

  feed.publish(now, frame(0, 1.0, 0.0));

  assert_eq!(feed.read(now + Duration::from_secs(1)), Some(frame(0, 1.0, 0.0)));
}

#[test]
fn draws_a_tick_behind_between_the_last_two_frames() {
  let feed: LoopFeed<TestFrame> = LoopFeed::default();
  let at: Instant = Instant::now();
  let step: Duration = Duration::from_millis(10);

  feed.publish(at, frame(1, 0.0, 100.0));
  feed.publish(at + step, frame(2, 1.0, 101.0));

  let read = |after: Duration| feed.read(at + step + after).expect("a frame was published");

  assert_eq!(read(Duration::ZERO), frame(1, 0.0, 100.0));
  assert!((read(Duration::from_millis(5)).state.0 - 0.5).abs() < 1e-5);
  assert!((read(Duration::from_millis(5)).time_of_day - 100.5).abs() < 1e-3);
  // A late tick holds the latest rather than running past it.
  assert_eq!(read(Duration::from_millis(30)), frame(2, 1.0, 101.0));
  assert_eq!(feed.get_latest(), Some(frame(2, 1.0, 101.0)));
}

#[test]
fn runs_the_day_forward_across_midnight_between_frames() {
  let feed: LoopFeed<TestFrame> = LoopFeed::default();
  let at: Instant = Instant::now();
  let step: Duration = Duration::from_millis(10);

  feed.publish(at, frame(1, 0.0, 86_399.0));
  feed.publish(at + step, frame(2, 0.0, 1.0));

  let time: f32 = feed.read(at + step + step / 2).expect("a frame").time_of_day;

  assert!(time.abs() < 1e-2, "{time} is not midnight");
}
