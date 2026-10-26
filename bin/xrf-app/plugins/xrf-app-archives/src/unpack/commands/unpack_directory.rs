use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobResource, JobStart};
use xrf_archive::ArchiveProject;
use xrf_job::JobProgress;
use xrf_pack::{ArchiveExtractOptions, ArchiveUnpackResult, ArchiveUnpacker};
use xrf_utils::format_path;

use crate::ipc::archives::ARCHIVES_JOB_UNPACK;
use crate::unpack::ArchivesUnpackRequest;

/// Unpack every archive of a directory into a destination tree, reporting progress and stopping on request.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "unpack_directory")]
pub async fn archives_unpack_directory(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: ArchivesUnpackRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<ArchiveUnpackResult> {
  let start: JobStart = JobStart::new(job_id, ARCHIVES_JOB_UNPACK)
    .with_request(&request)
    .with_kind_exclusion()
    .with_resources(vec![JobResource::tree(&request.destination)])
    .with_progress(progress);

  let ArchivesUnpackRequest {
    from: source,
    destination,
  } = request;

  log::info!(
    "Unpacking archive: {} -> {}",
    format_path(&source),
    format_path(&destination)
  );

  registry
    .run(
      &execution,
      start,
      move |job| {
        let project: ArchiveProject = ArchiveProject::new(&source)?;

        ArchiveUnpacker::unpack_opt(&project, &destination, ArchiveExtractOptions::default().with_job(job))
      },
      |result| result.outcome,
    )
    .await
}
