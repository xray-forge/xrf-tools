use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobResource, JobStart};
use xrf_app_core::session::SessionSnapshot;
use xrf_job::JobProgress;
use xrf_pack::{ArchiveExtractDirectoryResult, ArchiveExtractOptions};

use crate::browse::{ArchiveSubject, ArchiveSubjects, ArchivesExtractRequest};
use crate::ipc::archives::ARCHIVES_JOB_EXTRACT;

/// Write every file the open subject holds under one directory into a destination root.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "extract_directory"))]
#[tauri::command(rename = "extract_directory")]
pub async fn archives_extract_directory(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  subjects: State<'_, ArchiveSubjects>,
  request: ArchivesExtractRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<ArchiveExtractDirectoryResult> {
  let start: JobStart = JobStart::new(job_id, ARCHIVES_JOB_EXTRACT)
    .with_request(&request)
    .with_kind_exclusion()
    .with_resources(vec![JobResource::tree(&request.destination)])
    .with_progress(progress);

  let ArchivesExtractRequest {
    session_id,
    prefix,
    destination,
  } = request;

  log::info!("Extracting directory '{prefix}' to '{}'", destination.display());

  // Required before the job registers, so a closed subject never shows as a failed job.
  let subject: Arc<SessionSnapshot<ArchiveSubject>> = subjects.require(session_id)?;

  // An empty prefix means the whole tree, so this is a full unpack in everything but name.
  registry
    .run(
      &execution,
      start,
      move |job| subject.extract_directory(&prefix, &destination, ArchiveExtractOptions::default().with_job(job)),
      |result| result.outcome,
    )
    .await
}
