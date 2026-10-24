use std::sync::LazyLock;
use std::time::{Duration, Instant};

use xrf_utils::wall_clock_millis;

/// Wall clock at the moment the process began, in milliseconds since the epoch.
static STARTED_AT_EPOCH_MILLIS: LazyLock<u64> = LazyLock::new(wall_clock_millis);
/// Monotonic origin every uptime is measured from.
static STARTED_AT: LazyLock<Instant> = LazyLock::new(Instant::now);

/// When this process began, stamped from `main` so nothing in the assembly order can move the answer: the wall clock
/// for dates, the monotonic clock for elapsed time.
pub struct ProcessStart;

impl ProcessStart {
  /// Stamps the moment the process began.
  pub fn stamp() {
    LazyLock::force(&STARTED_AT_EPOCH_MILLIS);
    LazyLock::force(&STARTED_AT);
  }

  /// Milliseconds since the epoch at which this process began.
  pub fn get_started_at() -> u64 {
    *STARTED_AT_EPOCH_MILLIS
  }

  /// How long this process has been running.
  pub fn get_uptime() -> Duration {
    STARTED_AT.elapsed()
  }
}
