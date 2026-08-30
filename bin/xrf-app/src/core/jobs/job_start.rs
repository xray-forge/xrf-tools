use serde::Serialize;
use serde_json::Value;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::JobProgress;

/// What a command hands the registry to start a job.
///
/// A described start rather than a list of arguments: every field here is something only the calling command knows,
/// and three of the five are easy to swap for one another at a call site reading `(id, kind, keys, request, progress)`.
/// Named fields also mean the next thing a job has to carry is added without touching the commands that do not carry
/// it.
pub struct JobStart {
  /// Identity, minted by the frontend before the command is sent so a cancel can arrive before the job does.
  pub id: Uuid,
  /// What kind of work this is, and how a tool finds its own run again after its view was rebuilt.
  pub kind: String,
  /// What this job holds exclusively while it runs, so a second request for the same destination is refused.
  pub lease_keys: Vec<String>,
  /// What the job was asked to do, serialized by the command that knows the type and never read by the registry.
  ///
  /// Kept so a window that did not start the run can still say what is running: after a reload the arguments live
  /// nowhere else, and a bar with no subject is only marginally better than no bar.
  pub request: Option<Value>,
  /// The channel the calling page is watching on, and `None` where nothing is watching yet.
  pub progress: Option<Channel<JobProgress>>,
}

impl JobStart {
  /// A job of `kind` under `id`, holding nothing, describing nothing, watched by nobody.
  pub fn new(id: Uuid, kind: impl Into<String>) -> Self {
    Self {
      id,
      kind: kind.into(),
      lease_keys: Vec::new(),
      request: None,
      progress: None,
    }
  }

  /// Hold `lease_keys` for as long as the job runs.
  pub fn with_lease_keys(mut self, lease_keys: Vec<String>) -> Self {
    self.lease_keys = lease_keys;

    self
  }

  /// Describe what the job was asked to do.
  ///
  /// Serialized here rather than by the caller so a command hands over the arguments it already has. A request that
  /// cannot be serialized is dropped: it is a diagnostic, and refusing to start work over one would trade a job the
  /// user asked for against a label.
  pub fn with_request<T: Serialize>(mut self, request: &T) -> Self {
    self.request = serde_json::to_value(request)
      .inspect_err(|error| log::warn!("Job request was not retained: {error}"))
      .ok();

    self
  }

  /// Report progress to `progress` until somebody else attaches.
  pub fn with_progress(mut self, progress: Channel<JobProgress>) -> Self {
    self.progress = Some(progress);

    self
  }
}
