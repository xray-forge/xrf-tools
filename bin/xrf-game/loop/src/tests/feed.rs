use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::feed::loop_events::LoopEvents;
use crate::feed::loop_feed::LoopFeed;
use crate::feed::loop_frame::LoopFrame;
use crate::feed::loop_read::LoopRead;
use crate::feed::loop_status::LoopStatus;
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

  assert!(feed.read(now).is_none());

  feed.publish(now, frame(0, 1.0));

  let read: LoopRead<TestFrame> = feed.read(now + Duration::from_secs(1)).expect("a frame was published");

  assert_eq!(*read.latest, frame(0, 1.0));
  assert!(Arc::ptr_eq(&read.previous, &read.latest));
}

#[test]
fn reads_a_tick_behind_between_the_last_two_frames_sharing_them() {
  let feed: LoopFeed<TestFrame> = LoopFeed::default();
  let at: Instant = Instant::now();
  let step: Duration = Duration::from_millis(10);

  feed.publish(at, frame(1, 0.0));
  feed.publish(at + step, frame(2, 1.0));

  let read = |after: Duration| feed.read(at + step + after).expect("a frame was published");
  let halfway: LoopRead<TestFrame> = read(Duration::from_millis(5));

  assert_eq!(read(Duration::ZERO).alpha, 0.0);
  assert!((halfway.alpha - 0.5).abs() < 1e-5);
  assert_eq!(*halfway.previous, frame(1, 0.0));
  assert_eq!(*halfway.latest, frame(2, 1.0));
  assert_eq!(halfway.latest_at, at + step);
  // A late tick holds the latest rather than running past it.
  assert_eq!(read(Duration::from_millis(30)).alpha, 1.0);
  assert!(Arc::ptr_eq(&feed.get_latest().expect("the latest"), &halfway.latest));
}

#[test]
fn keeps_every_event_in_order_until_taken() {
  let events: LoopEvents<&str> = LoopEvents::default();

  events.push(3, ["step", "online"]);
  events.push(4, []);
  events.push(5, ["shot"]);

  let taken: Vec<(u64, &str)> = events
    .take()
    .into_iter()
    .map(|event| (event.tick, event.event))
    .collect();

  assert_eq!(taken, vec![(3, "step"), (3, "online"), (5, "shot")]);
  assert!(events.take().is_empty());
}

#[test]
fn keeps_the_first_way_the_loop_ended() {
  let feed: LoopFeed<TestFrame> = LoopFeed::default();
  let fault: LoopStatus = LoopStatus::Faulted {
    tick: 9,
    message: String::from("broken"),
  };

  assert_eq!(feed.get_status(), LoopStatus::Running);

  feed.end(fault.clone());
  feed.end(LoopStatus::Stopped);

  assert_eq!(feed.get_status(), fault);
}
