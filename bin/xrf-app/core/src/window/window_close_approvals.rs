use std::collections::HashSet;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// The windows whose page agreed to let them close, by label: what prepares a window for going acts only on these.
#[derive(Default)]
pub struct WindowCloseApprovals {
  approved: Mutex<HashSet<String>>,
}

impl WindowCloseApprovals {
  /// Lets the next close asked of `window` go ahead.
  pub fn approve(&self, window: &str) {
    self.lock().insert(window.to_owned());
  }

  /// Answers whether a close asked of `window` was approved, using the approval up.
  pub fn take(&self, window: &str) -> bool {
    self.lock().remove(window)
  }

  /// The approvals, whatever a panicking caller left: an entry is only ever added or removed whole.
  fn lock(&self) -> MutexGuard<'_, HashSet<String>> {
    self.approved.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
