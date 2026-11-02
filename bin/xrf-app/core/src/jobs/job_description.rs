use std::time::Duration;

use serde::Serialize;
use uuid::Uuid;
use xrf_job::JobProgress;

use crate::jobs::JobKind;
use crate::jobs::job_conclusion::JobConclusion;
#[cfg(feature = "typescript-bindings")]
use crate::jobs::opaque_json::OpaqueJson;

/// One job as the listing describes it, running or recently finished.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobDescription {
  pub id: Uuid,
  /// What kind of work this is, as the command that started it named itself.
  pub kind: JobKind,
  /// What this job holds exclusively, so a refused start can be explained by pointing at the job that refused it.
  pub lease_keys: Vec<String>,
  /// What the job was asked to do, as the command that started it described itself.
  /// Absent for a job whose command described nothing.
  #[cfg_attr(feature = "typescript-bindings", specta(type = Option<OpaqueJson>))]
  pub request: Option<serde_json::Value>,
  /// Whether stopping has been asked for. A job can carry this and still be running: cancellation lands at a boundary
  /// the operation chooses, and the gap between asking and stopping is exactly what a reader wants to see.
  pub is_cancel_requested: bool,
  /// The job's own progress: live for a running job, as last seen for a finished one.
  /// Absent for a job registered but not yet reporting — a run holding a lease while it validates its inputs, say.
  pub progress: Option<JobProgress>,
  /// Absent while the job is running.
  pub conclusion: Option<JobConclusion>,
  /// Why it failed, where it did.
  pub error: Option<String>,
  /// What the run answered, for a job that completed.
  /// Absent while the job runs, and for a job that failed or was cancelled before it had an answer.
  #[cfg_attr(feature = "typescript-bindings", specta(type = Option<OpaqueJson>))]
  pub result: Option<serde_json::Value>,
  /// How long the job ran, measured by the registry rather than by the operation.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
  /// When the registry took it on, as milliseconds since the epoch.
  pub started_at: u64,
}
