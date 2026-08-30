use tauri::ipc::Channel;
use xrf_job::{JobProgress, ProgressSink};

/// Sends a job's progress to the webview that asked for it.
///
/// One channel per run, constructed by the caller, so a superseded run's snapshots land on a callback the frontend has
/// already dropped. Stale-run isolation is a property of the transport here rather than a rule every consumer has to
/// remember to apply.
///
/// A send that fails is dropped rather than reported. The webview going away mid-run is ordinary — a reload, a closed
/// window — and it is not a reason to fail work that is writing files correctly.
pub struct ChannelProgressSink {
  channel: Channel<JobProgress>,
}

impl ChannelProgressSink {
  pub fn new(channel: Channel<JobProgress>) -> Self {
    Self { channel }
  }
}

impl ProgressSink for ChannelProgressSink {
  fn report(&self, progress: &JobProgress) {
    if let Err(error) = self.channel.send(progress.clone()) {
      log::debug!("Dropped a job progress update: {error}");
    }
  }
}
