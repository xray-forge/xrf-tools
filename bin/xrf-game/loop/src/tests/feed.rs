use std::time::{Duration, Instant};

use crate::feed::loop_feed::LoopFeed;
use crate::feed::loop_frame::LoopFrame;
use crate::tests::test_state::TestFrame;

fn frame(tick: u64, value: f32) -> LoopFrame<TestFrame> {
  LoopFrame {
    tick,
    state: TestFrame(value),
  }
}

#[test]
fn reads_nothing_before_the_first_frame_and_that_frame_alone_after_it() {
  let feed: LoopFeed<TestFrame> = LoopFeed::default();
  let now: Instant = Instant::now();

  assert_eq!(feed.read(now), None);

  feed.publish(now, frame(0, 1.0));

  assert_eq!(feed.read(now + Duration::from_secs(1)), Some(frame(0, 1.0)));
}

#[test]
fn draws_a_tick_behind_between_the_last_two_frames() {
  let feed: LoopFeed<TestFrame> = LoopFeed::default();
  let at: Instant = Instant::now();
  let step: Duration = Duration::from_millis(10);

  feed.publish(at, frame(1, 0.0));
  feed.publish(at + step, frame(2, 1.0));

  let read = |after: Duration| feed.read(at + step + after).expect("a frame was published");

  assert_eq!(read(Duration::ZERO), frame(1, 0.0));
  assert!((read(Duration::from_millis(5)).state.0 - 0.5).abs() < 1e-5);
  assert_eq!(read(Duration::from_millis(5)).tick, 2);
  // A late tick holds the latest rather than running past it.
  assert_eq!(read(Duration::from_millis(30)), frame(2, 1.0));
  assert_eq!(feed.get_latest(), Some(frame(2, 1.0)));
}
