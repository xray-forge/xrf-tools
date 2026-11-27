use std::time::{Duration, Instant};

use crate::clock::game_clock::GameClock;
use crate::clock::loop_clock::LoopClock;

#[test]
fn stands_each_tick_a_step_after_the_last() {
  let anchor: Instant = Instant::now();
  let clock: LoopClock = LoopClock::new(anchor, LoopClock::DEFAULT_TICK_RATE);

  assert_eq!(clock.get_step(), Duration::from_millis(10));
  assert_eq!(clock.get_instant(0), anchor);
  assert_eq!(clock.get_instant(250), anchor + Duration::from_millis(2500));
  assert_eq!(clock.get_tick_at(anchor + Duration::from_micros(29_999)), 2);
  assert_eq!(clock.get_tick_at(anchor + Duration::from_millis(30)), 3);
}

#[test]
fn takes_a_zero_rate_as_one_tick_a_second() {
  assert_eq!(LoopClock::new(Instant::now(), 0).get_step(), Duration::from_secs(1));
}

#[test]
fn gives_up_the_time_it_fell_behind() {
  let anchor: Instant = Instant::now();
  let mut clock: LoopClock = LoopClock::new(anchor, 100);
  let now: Instant = anchor + Duration::from_secs(5);

  clock.slip(40, now);

  assert_eq!(clock.get_instant(40), now);
  assert_eq!(clock.get_tick_at(now), 40);
}

#[test]
fn runs_the_day_from_its_start_across_midnight() {
  let clock: GameClock = GameClock::default();

  assert!((clock.get_time_of_day(0.0) - 9.0 * 3600.0).abs() < 1e-3);
  assert!((clock.get_time_of_day(16.0 * 3600.0) - 3600.0).abs() < 1e-3);
}
