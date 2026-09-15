use serde_json::Value;

use crate::core::jobs::job_conclusion::JobConclusion;

/// How a job ended, as the run itself reported it.
pub(super) struct JobEnding {
  pub(super) conclusion: JobConclusion,
  pub(super) error: Option<String>,
  /// What the run answered, serialized by the command that knows its type and never read here.
  pub(super) result: Option<Value>,
}
