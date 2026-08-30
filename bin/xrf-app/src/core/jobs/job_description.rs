use std::time::Duration;

use serde::Serialize;
use uuid::Uuid;
use xrf_job::JobProgress;

use crate::core::jobs::job_conclusion::JobConclusion;

/// One job as the listing describes it, running or recently finished.
///
/// One shape for both rather than two, because the panel showing them shows one list: a job crossing from running to
/// finished should change its fields, not its type. `conclusion` is what separates the halves.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobDescription {
  pub id: Uuid,
  /// What kind of work this is, as the command that started it named itself.
  pub kind: String,
  /// What this job holds exclusively, so a refused start can be explained by pointing at the job that refused it.
  pub lease_keys: Vec<String>,
  /// Whether stopping has been asked for. A job can carry this and still be running: cancellation lands at a boundary
  /// the operation chooses, and the gap between asking and stopping is exactly what a reader wants to see.
  pub is_cancel_requested: bool,
  /// The job's own progress: live for a running job, as last seen for a finished one.
  ///
  /// Absent for a job registered but not yet reporting — a run holding a lease while it validates its inputs, say.
  pub progress: Option<JobProgress>,
  /// Absent while the job is running.
  pub conclusion: Option<JobConclusion>,
  /// Why it failed, where it did.
  pub error: Option<String>,
  /// How long the job ran, measured by the registry rather than by the operation.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
}
