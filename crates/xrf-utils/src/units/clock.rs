use std::time::{SystemTime, UNIX_EPOCH};

use crate::units::duration_to_millis;

/// Read the wall clock, as milliseconds since the Unix epoch.
pub fn wall_clock_millis() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, duration_to_millis)
}

#[cfg(test)]
mod tests {
  use super::wall_clock_millis;

  /// Milliseconds at the start of 2020, which every clock this runs on is past.
  const YEAR_2020: u64 = 1_577_836_800_000;

  #[test]
  fn reads_the_wall_clock_and_not_an_arbitrary_origin() {
    // The distinction this exists for: a monotonic reading would be small and meaningless as a date.
    assert!(wall_clock_millis() > YEAR_2020);
  }

  #[test]
  fn does_not_go_backwards_between_two_reads() {
    assert!(wall_clock_millis() <= wall_clock_millis());
  }
}
