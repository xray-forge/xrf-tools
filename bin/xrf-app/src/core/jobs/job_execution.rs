use serde::Serialize;
use xrf_job::JobOutcome;

use crate::core::execution::ExecutionState;
use crate::core::jobs::JobRegistration;
use crate::core::types::TauriResult;

/// Runs registered work and retains its answer before releasing the job's leases.
///
/// The worker owns registration even if the awaiting command is dropped. `get_outcome` reads accepted cancellation
/// from the result; a late cancellation request cannot relabel completed work. Errors remain failures.
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
  E: ToString,
  F: FnOnce() -> Result<T, E> + Send + 'static,
{
  execution
    .run_blocking(what, move || {
      let result: TauriResult<T> = work().map_err(|error| error.to_string());
      let cancelled: bool = result
        .as_ref()
        .is_ok_and(|value| get_outcome(value) == JobOutcome::Cancelled);

      registration.conclude_with(&result, cancelled);

      result
    })
    .await?
}
