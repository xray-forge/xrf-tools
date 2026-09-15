use std::fmt::{Debug, Display, Formatter, Result as FormatterResult};
use std::sync::{Arc, Mutex, PoisonError};

use serde::Serialize;
use uuid::Uuid;

use crate::core::jobs::job_conclusion::JobConclusion;
use crate::core::jobs::job_ending::JobEnding;
use crate::core::jobs::job_registry::JobRegistry;

/// A registered job, and the leases it holds, for as long as this is alive.
pub struct JobRegistration {
  registry: Arc<JobRegistry>,
  id: Uuid,
  ending: Mutex<Option<JobEnding>>,
}

impl JobRegistration {
  /// The guard a registration hands back, ending unrecorded until the run says otherwise.
  pub(super) fn new(registry: Arc<JobRegistry>, id: Uuid) -> Self {
    Self {
      registry,
      id,
      ending: Mutex::new(None),
    }
  }

  pub fn id(&self) -> Uuid {
    self.id
  }

  /// Record how the job ended, before its leases are released.
  ///
  /// For a run with nothing to hand back. Where there is an answer, `conclude_with` keeps it.
  pub fn conclude(&self, conclusion: JobConclusion, error: Option<String>) {
    self.record(JobEnding {
      conclusion,
      error,
      result: None,
    });
  }

  /// Record the ending a `Result` describes, which is what a command has in hand.
  pub(super) fn conclude_with<T: Serialize, E: Display>(&self, outcome: &Result<T, E>, cancelled: bool) {
    let ending: JobEnding = match outcome {
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
    };

    self.record(ending);
  }

  fn record(&self, ending: JobEnding) {
    *self.ending.lock().unwrap_or_else(PoisonError::into_inner) = Some(ending);
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
    let ending: JobEnding = self
      .ending
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .take()
      .unwrap_or(JobEnding {
        conclusion: JobConclusion::Failed,
        error: None,
        result: None,
      });

    self.registry.finish(self.id, ending);
  }
}
