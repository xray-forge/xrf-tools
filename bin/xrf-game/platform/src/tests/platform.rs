use std::time::Duration;

use crate::platform::Platform;

#[test]
fn asks_sdl_for_a_millisecond_more_than_the_wait_so_a_short_one_never_returns_at_once() {
  assert_eq!(Platform::get_wait_milliseconds(Duration::ZERO), 0);
  assert_eq!(Platform::get_wait_milliseconds(Duration::from_micros(1)), 2);
  assert_eq!(Platform::get_wait_milliseconds(Duration::from_millis(1)), 2);
  assert_eq!(Platform::get_wait_milliseconds(Duration::from_micros(3_200)), 5);
  assert_eq!(Platform::get_wait_milliseconds(Duration::from_millis(4)), 5);
  assert_eq!(Platform::get_wait_milliseconds(Duration::MAX), u32::MAX);
}
