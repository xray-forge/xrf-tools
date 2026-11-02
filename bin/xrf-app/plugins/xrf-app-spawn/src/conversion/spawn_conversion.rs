use std::sync::Arc;

use serde::Serialize;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobKind, JobPhase, JobRegistry, JobResource, JobStart};
use xrf_chunk::XRayByteOrder;
use xrf_job::{JobHandle, JobOutcome, JobProgress};
use xrf_spawn::SpawnFile;
use xrf_utils::error_to_string;

use crate::SpawnConversionResult;
use crate::ipc::spawn::{SPAWN_JOB_PACK, SPAWN_JOB_UNPACK};
use crate::request::SpawnConvertRequest;

/// The conversion performed, retained with the result after a window reload.
#[derive(Clone, Copy, Debug, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum SpawnConversion {
  Pack,
  Unpack,
}

impl SpawnConversion {
  /// The lane both conversions share, including across windows.
  const LANE: &'static str = "spawn.convert";

  /// Runs the conversion as a job: registered before dispatch, holding its lane and destination until the blocking work
  /// actually ends.
  ///
  /// # Errors
  ///
  /// Returns why the job could not start, or why the conversion failed.
  pub async fn run(
    self,
    execution: &ExecutionState,
    registry: &Arc<JobRegistry>,
    request: SpawnConvertRequest,
    job_id: Uuid,
    progress: Channel<JobProgress>,
  ) -> TauriResult<SpawnConversionResult> {
    registry
      .run(
        execution,
        self.start(&request, job_id, progress),
        move |job| self.convert(&request, &job),
        |result| result.outcome,
      )
      .await
  }

  /// The job's place in the registry: its kind, its lane, and the destination it writes.
  pub(crate) fn start(self, request: &SpawnConvertRequest, job_id: Uuid, progress: Channel<JobProgress>) -> JobStart {
    JobStart::new(job_id, self.get_job_kind())
      .with_request(request)
      .with_exclusion_group(Self::LANE)
      .with_resources(vec![match self {
        Self::Pack => JobResource::file(&request.destination),
        Self::Unpack => JobResource::tree(&request.destination),
      }])
      .with_progress(progress)
  }

  /// Reads the source and writes the destination, giving up only before writing begins: the writers cannot stop
  /// safely partway through.
  pub(crate) fn convert(self, request: &SpawnConvertRequest, job: &JobHandle) -> TauriResult<SpawnConversionResult> {
    let mut result: SpawnConversionResult = SpawnConversionResult {
      operation: self,
      destination: request.destination.clone(),
      outcome: JobOutcome::Cancelled,
    };

    if job.is_cancelled() {
      return Ok(result);
    }

    let file: SpawnFile = {
      let _reading = job.enter(JobPhase::READ, None);

      job.set_detail(Some(request.source.display().to_string()));

      match self {
        Self::Pack => SpawnFile::import_from_path::<XRayByteOrder, _>(&request.source),
        Self::Unpack => SpawnFile::read_from_path::<XRayByteOrder, _>(&request.source),
      }
      .map_err(error_to_string)?
    };

    if job.is_cancelled() {
      return Ok(result);
    }

    // A late cancellation must not label completed output as cancelled.
    let _writing = job.enter(JobPhase::WRITE, None);

    job.set_detail(Some(request.destination.display().to_string()));

    match self {
      Self::Pack => file.write_to_path::<XRayByteOrder, _>(&request.destination),
      Self::Unpack => file.export_to_path::<XRayByteOrder, _>(&request.destination),
    }
    .map_err(error_to_string)?;

    result.outcome = JobOutcome::Completed;

    Ok(result)
  }

  fn get_job_kind(self) -> JobKind {
    match self {
      Self::Pack => SPAWN_JOB_PACK,
      Self::Unpack => SPAWN_JOB_UNPACK,
    }
  }
}

#[cfg(test)]
mod tests;
