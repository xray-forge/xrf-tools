use serde::Serialize;
use serde_json::Value;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::JobProgress;

use crate::jobs::{JobKind, JobResource};

/// What a command hands the registry to start a job.
pub struct JobStart {
  /// Identity, minted by the frontend before the command is sent so a cancel can arrive before the job does.
  pub(super) id: Uuid,
  /// What kind of work this is, and how a tool finds its own run again after its view was rebuilt.
  pub(super) kind: JobKind,
  /// What this job holds exclusively while it runs, so a second request for the same destination is refused.
  pub(super) resources: Vec<JobResource>,
  /// Action group held independently of destination leases, including read-only jobs and paired modes.
  pub(super) exclusion_group: Option<String>,
  /// What the job was asked to do, serialized by the command that knows the type and never read by the registry.
  pub(super) request: Option<Value>,
  /// The channel the calling page is watching on, and `None` where nothing is watching yet.
  pub(super) progress: Option<Channel<JobProgress>>,
}

impl JobStart {
  /// A job of `kind` under `id`, holding nothing, describing nothing, watched by nobody.
  pub fn new(id: Uuid, kind: JobKind) -> Self {
    Self {
      id,
      kind,
      resources: Vec::new(),
      exclusion_group: None,
      request: None,
      progress: None,
    }
  }

  /// Hold `resources` for as long as the job runs.
  pub fn with_resources(mut self, resources: Vec<JobResource>) -> Self {
    self.resources = resources;

    self
  }

  /// Hold the kind's own group across all windows until the job settles, so one run of the kind goes at a time.
  pub fn with_kind_exclusion(self) -> Self {
    let kind: JobKind = self.kind;

    self.with_exclusion_group(kind.as_str())
  }

  /// Hold `group` across all windows until the job settles, for kinds that share a lane.
  pub fn with_exclusion_group(mut self, group: impl Into<String>) -> Self {
    self.exclusion_group = Some(group.into());

    self
  }

  /// Describe what the job was asked to do.
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

  /// The identity the job registers under.
  pub fn get_id(&self) -> Uuid {
    self.id
  }

  /// The kind of work the job is.
  pub fn get_kind(&self) -> JobKind {
    self.kind
  }
}
