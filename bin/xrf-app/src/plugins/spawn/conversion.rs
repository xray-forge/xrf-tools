use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_db::{SpawnFile, XRayByteOrder};
use xrf_job::{JobHandle, JobOutcome, JobProgress};

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::spawn::request::SpawnConversionRequest;

/// Both conversions share one writer lane, including across windows.
const CONVERSION_LEASE: &str = "spawn.convert";

/// The conversion performed, retained with the result after a window reload.
#[derive(Clone, Copy, Debug, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum SpawnConversion {
  Pack,
  Unpack,
}

impl SpawnConversion {
  fn job_kind(self) -> &'static str {
    match self {
      Self::Pack => "spawn.pack",
      Self::Unpack => "spawn.unpack",
    }
  }
}

/// What reached disk; cancellation is accepted only before writing begins.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct SpawnConversionResult {
  pub operation: SpawnConversion,
  pub destination: PathBuf,
  pub outcome: JobOutcome,
}

/// Registers before dispatch and holds the lease until the blocking work actually ends.
pub async fn run_conversion(
  execution: &ExecutionState,
  registry: &Arc<JobRegistry>,
  request: SpawnConversionRequest,
  operation: SpawnConversion,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<SpawnConversionResult> {
  let (job, registration): (JobHandle, JobRegistration) =
    register_conversion(registry, &request, operation, job_id, progress)?;

  execution
    .run_blocking("Spawn conversion", move || {
      let outcome: TauriResult<SpawnConversionResult> = convert(&request, operation, &job);
      let cancelled: bool = outcome
        .as_ref()
        .is_ok_and(|result| result.outcome == JobOutcome::Cancelled);

      registration.conclude_with(&outcome, cancelled);

      outcome
    })
    .await?
}

/// Takes the conversion lease and the job's place in the registry, before any work is dispatched.
fn register_conversion(
  registry: &Arc<JobRegistry>,
  request: &SpawnConversionRequest,
  operation: SpawnConversion,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<(JobHandle, JobRegistration)> {
  registry.register(
    JobStart::new(job_id, operation.job_kind())
      .with_lease_keys(vec![CONVERSION_LEASE.to_owned()])
      .with_request(request)
      .with_progress(progress),
  )
}

fn convert(
  request: &SpawnConversionRequest,
  operation: SpawnConversion,
  job: &JobHandle,
) -> TauriResult<SpawnConversionResult> {
  let mut result: SpawnConversionResult = SpawnConversionResult {
    operation,
    destination: request.destination.clone(),
    outcome: JobOutcome::Cancelled,
  };

  if job.is_cancelled() {
    return Ok(result);
  }

  let file: SpawnFile = {
    let _reading = job.enter("read", None);

    job.set_detail(Some(request.source.display().to_string()));

    match operation {
      SpawnConversion::Pack => SpawnFile::import_from_path::<XRayByteOrder, _>(&request.source),
      SpawnConversion::Unpack => SpawnFile::read_from_path::<XRayByteOrder, _>(&request.source),
    }
    .map_err(error_to_string)?
  };

  if job.is_cancelled() {
    return Ok(result);
  }

  // The writers cannot stop safely partway through. A late cancellation must not label completed output as cancelled.
  let _writing = job.enter("write", None);

  job.set_detail(Some(request.destination.display().to_string()));

  match operation {
    SpawnConversion::Pack => file.write_to_path::<XRayByteOrder, _>(&request.destination),
    SpawnConversion::Unpack => file.export_to_path::<XRayByteOrder, _>(&request.destination),
  }
  .map_err(error_to_string)?;

  result.outcome = JobOutcome::Completed;

  Ok(result)
}

#[cfg(test)]
mod tests;
