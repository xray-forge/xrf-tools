use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Serialize;
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_archive::ArchiveProject;
use xrf_job::{JobHandle, JobProgress};
use xrf_pack::{ArchiveUnpackOptions, ArchiveUnpackResult, ArchiveUnpacker};

use crate::core::error::error_to_string;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::archives::lease::to_unpack_lease_key;

/// What an unpack was asked to do, for a window that has to describe a run it did not start.
///
/// Declared here rather than reusing the command's own parameters because the pair is what a reader needs and the
/// parameters are `&str` handles to it. Serialized into the job listing and read by nothing on this side.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ArchiveUnpackRequest<'paths> {
  source: &'paths Path,
  destination: &'paths Path,
}

/// Unpack every archive of a directory into a destination tree, reporting progress and stopping on request.
///
/// A cancelled run answers with a result rather than an error. It leaves the files it had already written where they
/// are — deleting them is not an option, because the destination may have held the user's own files and nothing here
/// can tell those apart from this run's — so the caller needs the counts to say what is now on disk.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "unpack_directory"))]
#[tauri::command(rename = "unpack_directory")]
pub async fn archives_unpack_directory(
  registry: State<'_, Arc<JobRegistry>>,
  from: &str,
  destination: &str,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<ArchiveUnpackResult> {
  log::info!("Open archive directory: {}", from);
  log::info!("Unpacking archive to: {}", destination);

  let source: PathBuf = PathBuf::from(from);
  let destination: PathBuf = PathBuf::from(destination);
  // Before the hop, never inside it: registering in the blocking closure leaves a window where a second request sees
  // no holder and both write the same tree.
  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, "archives.unpack")
      .with_lease_keys(vec![to_unpack_lease_key(&destination)])
      .with_request(&ArchiveUnpackRequest {
        source: &source,
        destination: &destination,
      })
      .with_progress(progress),
  )?;

  // Off the async worker, indexing included: reading the volumes, decompressing every entry, and writing the tree are
  // all synchronous, and the unpacker drives a pool of its own rather than yielding to this one.
  let unpacking: JobHandle = job.clone();
  let outcome: TauriResult<ArchiveUnpackResult> = tauri::async_runtime::spawn_blocking(move || {
    let project: ArchiveProject = ArchiveProject::new(&source)?;

    ArchiveUnpacker::unpack_opt(
      &project,
      &destination,
      ArchiveUnpackOptions::default().with_job(unpacking),
    )
  })
  .await
  .map_err(|error| format!("Archive unpack did not finish: {error}"))?
  .map_err(error_to_string);

  registration.conclude_with(&outcome, job.is_cancelled());

  outcome
}
