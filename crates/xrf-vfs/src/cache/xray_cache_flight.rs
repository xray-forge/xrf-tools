use std::sync::{Condvar, Mutex, MutexGuard};
use std::thread::{self, ThreadId};

/// One load in progress, and the thread performing it.
#[derive(Debug)]
pub(crate) struct XrayCacheFlight {
  pub(super) owner: ThreadId,
  is_finished: Mutex<bool>,
  finished: Condvar,
}

impl XrayCacheFlight {
  /// Blocks until the owner is done, returning at once when it already is.
  ///
  /// Safe against a finish that lands between a caller receiving this flight and waiting on it: the owner sets the flag
  /// before notifying, and a waiter that arrives afterwards sees the flag rather than a notification it missed.
  pub(crate) fn wait(&self) {
    let mut is_finished = self.lock();

    while !*is_finished {
      is_finished = self.finished.wait(is_finished).expect("cache flight is never poisoned");
    }
  }

  pub(super) fn new() -> Self {
    Self {
      owner: thread::current().id(),
      is_finished: Mutex::new(false),
      finished: Condvar::new(),
    }
  }

  pub(super) fn finish(&self) {
    *self.lock() = true;

    self.finished.notify_all();
  }

  fn lock(&self) -> MutexGuard<'_, bool> {
    self.is_finished.lock().expect("cache flight is never poisoned")
  }
}
