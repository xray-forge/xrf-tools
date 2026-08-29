use std::sync::{Arc, Mutex, MutexGuard};

use xrf_archive::ArchiveProject;

use crate::core::types::TauriResult;

/// The archive explorer's open project, shared by every command that reads it.
pub struct ArchiveProjectState {
  /// Held behind an `Arc` so a reader can take a snapshot and let the lock go. Work bounded only by archive size runs
  /// on a blocking thread, which cannot borrow a guard, and cloning the index itself per call would copy every entry.
  pub project: Mutex<Option<Arc<ArchiveProject>>>,
}

impl ArchiveProjectState {
  pub fn new() -> Self {
    Self {
      project: Mutex::new(None),
    }
  }

  /// The open project as a handle the caller owns, naming the attempted action in either failure.
  ///
  /// The lock is held only long enough to clone the handle. Holding it across archive work would move the starvation
  /// this snapshot exists to avoid off the executor and onto this mutex, where `has_project` would queue behind it.
  ///
  /// A snapshot also outlives the state it came from: closing the archive while an extraction runs leaves that run
  /// writing the entries it was asked for, rather than failing halfway through a tree it already started.
  pub fn require(&self, action: &str) -> TauriResult<Arc<ArchiveProject>> {
    let lock: MutexGuard<Option<Arc<ArchiveProject>>> = self
      .project
      .lock()
      .map_err(|error| format!("Failed to {action} - archive state is unavailable: {error}"))?;

    lock
      .as_ref()
      .cloned()
      .ok_or_else(|| format!("Failed to {action} - archive is not open"))
  }
}
