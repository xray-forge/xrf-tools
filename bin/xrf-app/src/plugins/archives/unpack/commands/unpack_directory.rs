use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_archive::ArchiveProject;
use xrf_job::{JobHandle, JobProgress};
use xrf_pack::{ArchiveUnpackOptions, ArchiveUnpackResult, ArchiveUnpacker};

use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobResource, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::archives::unpack::ArchivesUnpackRequest;

/// Unpack every archive of a directory into a destination tree, reporting progress and stopping on request.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "unpack_directory"))]
#[tauri::command(rename = "unpack_directory")]
pub async fn archives_unpack_directory(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: ArchivesUnpackRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<ArchiveUnpackResult> {
  let start: JobStart = JobStart::new(job_id, JobKind::ArchivesUnpack).with_request(&request);

  let ArchivesUnpackRequest {
    from: source,
    destination,
  } = request;

  log::info!("Open archive directory: {}", source.display());
  log::info!("Unpacking archive to: {}", destination.display());

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    start
      .with_exclusion_group(JobKind::ArchivesUnpack.as_str())
      .with_resources(vec![JobResource::tree(&destination)])
      .with_progress(progress),
  )?;

  run_job(
    &execution,
    "Archive unpack",
    registration,
    move || {
      let project: ArchiveProject = ArchiveProject::new(&source)?;

      ArchiveUnpacker::unpack_opt(&project, &destination, ArchiveUnpackOptions::default().with_job(job))
    },
    |result| result.outcome,
  )
  .await
}
