//! When this process began, stamped from `main` so nothing in the assembly order can move the answer.

use std::sync::LazyLock;
use std::time::{Duration, Instant};

use xrf_utils::wall_clock_millis;

/// Wall clock at the moment the process began, in milliseconds since the epoch.
///
/// The only one of the two readings that can be rendered as a date.
static PROCESS_STARTED_AT_EPOCH_MILLIS: LazyLock<u64> = LazyLock::new(wall_clock_millis);

/// Monotonic origin every uptime is measured from.
///
/// Separate from the reading above because the wall clock can be set backwards between two reads, which would show a
/// running application as not yet started. Neither clock answers the other's question, so both are stamped.
static PROCESS_STARTED_AT: LazyLock<Instant> = LazyLock::new(Instant::now);

/// Stamps the moment the process began.
///
/// Called first thing in `main`, and both readings together so they describe the same moment. Without it each would be
/// stamped by whichever caller first asked, and an application running for an hour would report that it had just
/// started.
pub fn setup_process_start() {
  LazyLock::force(&PROCESS_STARTED_AT_EPOCH_MILLIS);
  LazyLock::force(&PROCESS_STARTED_AT);
}

/// Milliseconds since the epoch at which this process began.
pub fn process_started_at_epoch_millis() -> u64 {
  *PROCESS_STARTED_AT_EPOCH_MILLIS
}

/// How long this process has been running.
///
/// Answers the elapsed time rather than the origin, so no caller has to know which of the two clocks measures it.
pub fn process_uptime() -> Duration {
  PROCESS_STARTED_AT.elapsed()
}
