use std::time::{Duration, Instant};

use crate::frame::timing::frame_clock::FrameClock;

#[test]
fn frames_draw_at_the_wall_clock_until_stepped_then_a_step_a_frame() {
  let start: Instant = Instant::now();
  let step: Duration = Duration::from_millis(16);
  let mut clock: FrameClock = FrameClock::default();

  assert_eq!(clock.tick(start), start);
  assert_eq!(
    clock.tick(start + Duration::from_secs(1)),
    start + Duration::from_secs(1)
  );

  clock.set_step(Some(step));

  // However long a frame takes, the next is drawn a step after the last.
  assert_eq!(
    clock.tick(start + Duration::from_secs(9)),
    start + Duration::from_secs(1) + step
  );
  assert_eq!(
    clock.tick(start + Duration::from_secs(10)),
    start + Duration::from_secs(1) + step * 2
  );

  clock.set_step(None);

  assert_eq!(
    clock.tick(start + Duration::from_secs(11)),
    start + Duration::from_secs(11)
  );
}

#[test]
fn a_step_past_the_platforms_latest_time_holds_the_clock() {
  let start: Instant = Instant::now();
  let mut clock: FrameClock = FrameClock::default();

  clock.tick(start);
  clock.set_step(Some(Duration::MAX));

  assert_eq!(clock.tick(start), start);
  assert_eq!(clock.tick(start), start);
}
