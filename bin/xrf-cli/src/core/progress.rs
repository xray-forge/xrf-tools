use std::sync::Arc;
use std::time::Duration;

use xrf_job::{JobHandle, LoggingSink};

/// How often a long command says where it has got to.
///
/// Coarse next to what a window would use. These are log lines in a terminal, and a run reporting ten times a second
/// would bury everything it said before.
const PROGRESS_INTERVAL: Duration = Duration::from_secs(2);

/// A job that reports as log lines and is never cancelled.
///
/// Nothing here can stop a run: the command line installs no signal handler, so the handle is passed for its reporting
/// alone. It is a real handle rather than an inert one because a command that walks a whole project owes a terminal
/// some sign that it is still moving.
pub fn new_logging_job() -> JobHandle {
  JobHandle::with_interval(Arc::new(LoggingSink), PROGRESS_INTERVAL)
}
