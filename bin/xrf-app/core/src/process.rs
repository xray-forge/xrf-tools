//! When this process began, stamped from `main` so nothing in the assembly order can move the answer.

use std::sync::LazyLock;
use std::time::{Duration, Instant};

use xrf_utils::wall_clock_millis;

/// Wall clock at the moment the process began, in milliseconds since the epoch.
static PROCESS_STARTED_AT_EPOCH_MILLIS: LazyLock<u64> = LazyLock::new(wall_clock_millis);
/// Monotonic origin every uptime is measured from.
static PROCESS_STARTED_AT: LazyLock<Instant> = LazyLock::new(Instant::now);

/// Stamps the moment the process began.
pub fn setup_process_start() {
  LazyLock::force(&PROCESS_STARTED_AT_EPOCH_MILLIS);
  LazyLock::force(&PROCESS_STARTED_AT);
}

/// Milliseconds since the epoch at which this process began.
pub fn process_started_at_epoch_millis() -> u64 {
  *PROCESS_STARTED_AT_EPOCH_MILLIS
}

/// How long this process has been running.
pub fn process_uptime() -> Duration {
  PROCESS_STARTED_AT.elapsed()
}
