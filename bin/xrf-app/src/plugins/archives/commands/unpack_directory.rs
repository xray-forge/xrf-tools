use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_archive::ArchiveProject;
use xrf_job::{JobHandle, JobProgress};
use xrf_pack::{ArchiveUnpackOptions, ArchiveUnpackResult, ArchiveUnpacker};

use crate::core::error::error_to_string;
use crate::core::jobs::{ChannelProgressSink, JobRegistration, JobRegistry};
use crate::core::types::TauriResult;
use crate::plugins::archives::lease::to_unpack_lease_key;

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
  let job: JobHandle = JobHandle::new(Arc::new(ChannelProgressSink::new(progress)));

  // Before the hop, never inside it: registering in the blocking closure leaves a window where a second request sees
  // no holder and both write the same tree.
  let registration: JobRegistration = registry.register(
    job_id,
    "archives.unpack",
    vec![to_unpack_lease_key(&destination)],
    job.clone(),
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
