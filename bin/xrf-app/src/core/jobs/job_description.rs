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
  /// What the job was asked to do, as the command that started it described itself.
  ///
  /// JSON for the same reason the answer is: the registry serves every domain and reads none of their argument types.
  /// It is what lets a window that did not start a run still name what is running.
  ///
  /// Absent for a job whose command described nothing.
  #[cfg_attr(feature = "typescript-bindings", specta(type = Option<OpaqueJson>))]
  pub request: Option<serde_json::Value>,
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
  /// What the run answered, for a job that completed.
  ///
  /// JSON rather than a type, because the registry serves every domain and none of their result types are its
  /// business. The tool that started the work is the one that knows how to read it.
  ///
  /// Absent while the job runs, and for a job that failed or was cancelled before it had an answer.
  #[cfg_attr(feature = "typescript-bindings", specta(type = Option<OpaqueJson>))]
  pub result: Option<serde_json::Value>,
  /// How long the job ran, measured by the registry rather than by the operation.
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
}

/// The Typescript face of a value this application does not describe: `unknown`.
///
/// `serde_json::Value` cannot be mirrored directly - Specta inlines it, and a self-referential inline type has no
/// finite Typescript expansion. Naming it `unknown` says the same thing the Rust side says, and better than a shape
/// would: whoever reads the field is the one that knows what is in it, and a caller that guesses has to narrow first.
#[cfg(feature = "typescript-bindings")]
pub struct OpaqueJson;

#[cfg(feature = "typescript-bindings")]
impl specta::Type for OpaqueJson {
  fn definition(_: &mut specta::Types) -> specta::datatype::DataType {
    specta::datatype::DataType::Reference(specta_typescript::define("unknown"))
  }
}
