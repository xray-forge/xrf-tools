use std::time::Duration;

use crate::level::engine_clock::EngineClock;

fn millis(value: u64) -> Duration {
  Duration::from_millis(value)
}

/// The engine's clock adds up the level's a frame at a time: not at all on its first step, each step at most the
/// engine's longest, and none where the level's clock went back, going on from there; so a burst at a time of its own
/// and the frames' clock taking over again after are each one short step to what it drives.
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

  // A burst jumps the level's clock forward to its own time: one longest step.
  assert_eq!(clock.advance(millis(600_000)), millis(16) + longest);
  assert_eq!(clock.advance(millis(600_500)), millis(16) + longest * 2);

  // The frames' clock takes over again behind it: no step back, then on from there.
  assert_eq!(clock.advance(millis(15_000)), millis(16) + longest * 2);
  assert_eq!(clock.advance(millis(15_016)), millis(32) + longest * 2);
}
