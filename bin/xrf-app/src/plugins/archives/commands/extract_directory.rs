use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_archive::ArchiveProject;
use xrf_job::{JobHandle, JobProgress};
use xrf_pack::{ArchiveExtractDirectoryResult, ArchiveExtractOptions, ArchiveUnpacker};

use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobResource, JobStart, run_job};
use crate::core::session::SessionSnapshot;
use crate::core::types::TauriResult;
use crate::plugins::archives::request::ArchivesExtractRequest;
use crate::plugins::archives::state::ArchiveProjectState;

/// Write every archived file under one directory into a destination root.
///
/// An empty prefix means the whole archive, so this also covers extracting everything without needing
/// a separate command — which is why it is a job rather than a quick read.
///
/// Holds the destination tree exclusively, sharing that lease with an unpack: both lay the archive's own layout into
/// the root, so two runs there overlap whatever each was asked for, even where their prefixes differ.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "extract_directory"))]
#[tauri::command(rename = "extract_directory")]
pub async fn archives_extract_directory(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  state: State<'_, ArchiveProjectState>,
  request: ArchivesExtractRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<ArchiveExtractDirectoryResult> {
  let start: JobStart = JobStart::new(job_id, JobKind::ArchivesExtract).with_request(&request);

  let ArchivesExtractRequest {
    session_id,
    prefix,
    destination,
  } = request;

  log::info!(
    "Extracting archive directory '{}' to '{}'",
    prefix,
    destination.display()
  );

  let project: Arc<SessionSnapshot<ArchiveProject>> = state.require(session_id)?;
  let prefix: String = prefix.to_owned();

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    start
      .with_exclusion_group(JobKind::ArchivesExtract.as_str())
      .with_resources(vec![JobResource::tree(&destination)])
      .with_progress(progress),
  )?;

  // Off the async worker: an empty prefix means the whole archive, so this is a full unpack in everything but name.
  run_job(
    &execution,
    "Archive directory extraction",
    registration,
    move || {
      ArchiveUnpacker::extract_directory_opt(
        &project,
        &prefix,
        &destination,
        ArchiveExtractOptions::default().with_job(job),
      )
    },
    |result| result.outcome,
  )
  .await
}
