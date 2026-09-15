use std::fmt::Display;

use serde::Serialize;
use xrf_job::JobOutcome;

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::JobRegistration;
use crate::core::types::TauriResult;

/// Runs registered work and retains its answer before releasing the job's leases.
///
/// # Errors
///
/// Returns the work's error or a blocking execution failure.
pub async fn run_job<T, E, F>(
  execution: &ExecutionState,
  what: &str,
  registration: JobRegistration,
  work: F,
  get_outcome: fn(&T) -> JobOutcome,
) -> TauriResult<T>
where
  T: Serialize + Send + 'static,
  E: Display,
  F: FnOnce() -> Result<T, E> + Send + 'static,
{
  execution
    .run_blocking(what, move || {
      let result: TauriResult<T> = work().map_err(error_to_string);

      let cancelled: bool = result
        .as_ref()
        .is_ok_and(|value| get_outcome(value) == JobOutcome::Cancelled);

      registration.conclude_with(&result, cancelled);

      result
    })
    .await?
}
