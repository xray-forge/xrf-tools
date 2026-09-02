use std::path::PathBuf;
use std::sync::Arc;

use serde_json::json;
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_archive::ArchiveProject;
use xrf_job::{JobHandle, JobProgress};
use xrf_pack::{ArchiveExtractDirectoryResult, ArchiveExtractOptions, ArchiveUnpacker};

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::archives::lease::{EXTRACT_JOB_KIND, to_destination_tree_lease_key};
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
  prefix: &str,
  destination: &str,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<ArchiveExtractDirectoryResult> {
  log::info!("Extracting archive directory '{}' to '{}'", prefix, destination);

  let project: Arc<ArchiveProject> = state.require("extract directory")?;
  let prefix: String = prefix.to_owned();
  let destination: PathBuf = PathBuf::from(destination);

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, EXTRACT_JOB_KIND)
      .with_lease_keys(vec![to_destination_tree_lease_key(&destination)])
      .with_request(&json!({ "prefix": prefix, "destination": destination }))
      .with_progress(progress),
  )?;

  // Off the async worker: an empty prefix means the whole archive, so this is a full unpack in everything but name.
  let extracting: JobHandle = job.clone();
  let outcome: TauriResult<ArchiveExtractDirectoryResult> = execution
    .run_blocking("Archive directory extraction", move || {
      ArchiveUnpacker::extract_directory_opt(
        &project,
        &prefix,
        &destination,
        ArchiveExtractOptions::default().with_job(extracting),
      )
    })
    .await?
    .map_err(error_to_string);

  registration.conclude_with(&outcome, job.is_cancelled());

  outcome
}
