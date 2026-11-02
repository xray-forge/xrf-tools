use std::num::NonZeroUsize;

use serde::Serialize;
use xrf_job::ExecutionPlan;

/// The width a run was bounded to.
///
/// Named for the policy rather than for the `--jobs` flag that sets it: a `job` in this workspace is a tracked unit of
/// work with progress and cancellation, and a field called `jobs` holding a worker count would be that word meaning
/// something else in the one document both concepts could appear in.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandExecution {
  workers: NonZeroUsize,
}

impl From<&ExecutionPlan> for CommandExecution {
  fn from(plan: &ExecutionPlan) -> Self {
    Self {
      workers: plan.get_workers(),
    }
  }
}
