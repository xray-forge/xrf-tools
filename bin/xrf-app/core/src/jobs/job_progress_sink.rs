use std::sync::{PoisonError, RwLock, RwLockReadGuard};

use tauri::ipc::Channel;
use xrf_job::{JobProgress, ProgressSink};

/// Sends a job's progress to whichever webview is currently watching it.
pub struct JobProgressSink {
  channel: RwLock<Option<Channel<JobProgress>>>,
}

impl JobProgressSink {
  /// A sink reporting to `channel`, which is what a command that was called by a page starts with.
  pub fn new(channel: Channel<JobProgress>) -> Self {
    Self {
      channel: RwLock::new(Some(channel)),
    }
  }

  /// A sink reporting nowhere, for a job nobody is watching yet.
  pub fn detached() -> Self {
    Self {
      channel: RwLock::new(None),
    }
  }

  /// Report to `channel` from now on, dropping whatever was there.
  pub fn attach(&self, channel: Channel<JobProgress>) {
    *self.channel.write().unwrap_or_else(PoisonError::into_inner) = Some(channel);
  }

  /// A poisoned lock still names a live channel, and refusing to report because a snapshot once panicked would leave
  /// the watcher with a frozen bar over a job that is running perfectly well.
  fn read(&self) -> RwLockReadGuard<'_, Option<Channel<JobProgress>>> {
    self.channel.read().unwrap_or_else(PoisonError::into_inner)
  }
}

impl ProgressSink for JobProgressSink {
  fn report(&self, progress: &JobProgress) {
    if let Some(channel) = self.read().as_ref()
      && let Err(error) = channel.send(progress.clone())
    {
      log::debug!("Dropped a job progress update: {error}");
    }
  }
}
