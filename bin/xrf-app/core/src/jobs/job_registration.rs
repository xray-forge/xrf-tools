use std::fmt::{Debug, Display, Formatter, Result as FormatterResult};
use std::sync::Arc;

use serde::Serialize;
use uuid::Uuid;

use crate::jobs::job_conclusion::JobConclusion;
use crate::jobs::job_ending::JobEnding;
use crate::jobs::job_registry::JobRegistry;

/// A registered job, and the leases it holds, for as long as this is alive.
pub(crate) struct JobRegistration {
  registry: Arc<JobRegistry>,
  id: Uuid,
  ending: Option<JobEnding>,
}

impl JobRegistration {
  /// The guard a registration hands back, ending unrecorded until the run says otherwise.
  pub(super) fn new(registry: Arc<JobRegistry>, id: Uuid) -> Self {
    Self {
      registry,
      id,
      ending: None,
    }
  }

  /// Record how the job ended, before its leases are released.
  ///
  /// For a run with nothing to hand back. Where there is an answer, `conclude_with` keeps it.
  #[cfg(test)]
  pub(super) fn conclude(&mut self, conclusion: JobConclusion, error: Option<String>) {
    self.ending = Some(JobEnding {
      conclusion,
      error,
      result: None,
    });
  }

  /// Record the ending a `Result` describes, which is what a command has in hand.
  pub(super) fn conclude_with<T: Serialize, E: Display>(&mut self, outcome: &Result<T, E>, cancelled: bool) {
    self.ending = Some(match outcome {
      Ok(value) => JobEnding {
        conclusion: if cancelled {
          JobConclusion::Cancelled
        } else {
          JobConclusion::Completed
        },
        error: None,
        result: serde_json::to_value(value)
          .inspect_err(|error| log::warn!("Job answer was not retained: {error}"))
          .ok(),
      },
      Err(error) => JobEnding {
        conclusion: JobConclusion::Failed,
        error: Some(error.to_string()),
        result: None,
      },
    });
  }
}

impl Debug for JobRegistration {
  /// The identity and nothing else: the registry behind this is shared mutable state, and rendering it would mean
  /// taking a lock inside a formatter, which is how a diagnostic turns into a deadlock.
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatterResult {
    formatter.debug_struct("JobRegistration").field("id", &self.id).finish()
  }
}

impl Drop for JobRegistration {
  /// Releases the leases and retains the job, concluding it as failed where nobody said otherwise.
  fn drop(&mut self) {
    let ending: JobEnding = self.ending.take().unwrap_or(JobEnding {
      conclusion: JobConclusion::Failed,
      error: None,
      result: None,
    });

    self.registry.finish(self.id, ending);
  }
}
