use std::cell::Cell;
use std::io::{Error as IoError, ErrorKind};

thread_local! {
  /// Per-thread, so tests running in parallel cannot arm each other's failure.
  static NEXT_WRITE_FAILS: Cell<bool> = const { Cell::new(false) };
  /// How many publishes on this thread succeed before one fails, when armed.
  static PUBLISHES_BEFORE_FAILURE: Cell<Option<u32>> = const { Cell::new(None) };
}

/// The publication seam a test fails on purpose.
pub struct StagingFaults;

impl StagingFaults {
  /// Arms the next staged write on this thread to fail as a full device would.
  pub fn fail_next_write() {
    NEXT_WRITE_FAILS.with(|armed| armed.set(true));
  }

  /// Arms a publish on this thread to fail once `succeeding` more have succeeded, as a rename refused by a locked file
  /// would.
  pub fn fail_publish_after(succeeding: u32) {
    PUBLISHES_BEFORE_FAILURE.with(|armed| armed.set(Some(succeeding)));
  }

  /// Consumes the arming, so one write fails and the next stages normally.
  pub(crate) fn take_write_failure() -> Option<IoError> {
    NEXT_WRITE_FAILS
      .with(|armed| armed.replace(false))
      .then(|| IoError::new(ErrorKind::StorageFull, "Injected staging failure"))
  }

  /// Counts one publish down, failing it where the arming has run out.
  pub(crate) fn take_publish_failure() -> Option<IoError> {
    PUBLISHES_BEFORE_FAILURE.with(|armed| match armed.get() {
      Some(0) => {
        armed.set(None);

        Some(IoError::new(ErrorKind::PermissionDenied, "Injected publish failure"))
      }
      Some(remaining) => {
        armed.set(Some(remaining - 1));

        None
      }
      None => None,
    })
  }
}
