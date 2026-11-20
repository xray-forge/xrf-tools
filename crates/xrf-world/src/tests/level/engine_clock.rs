use std::time::Duration;

use crate::level::engine_clock::EngineClock;

fn millis(value: u64) -> Duration {
  Duration::from_millis(value)
}

#[test]
fn adds_up_the_levels_clock_a_frame_at_a_time_never_back() {
  let mut clock: EngineClock = EngineClock::default();
  let longest: Duration = EngineClock::LONGEST_STEP;

  assert_eq!(
    clock.advance(millis(12_000)),
    Duration::ZERO,
    "its first step stands still"
  );
  assert_eq!(clock.advance(millis(12_016)), millis(16));

  // A burst jumps the level's clock forward.
  assert_eq!(clock.advance(millis(600_000)), millis(16) + longest);
  assert_eq!(clock.advance(millis(600_500)), millis(16) + longest * 2);

  // The frames' clock takes over again behind it.
  assert_eq!(clock.advance(millis(15_000)), millis(16) + longest * 2);
  assert_eq!(clock.advance(millis(15_016)), millis(32) + longest * 2);
}
