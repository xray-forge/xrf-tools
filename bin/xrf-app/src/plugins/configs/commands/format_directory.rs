use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_ltx::{LtxFormatOptions, LtxProject, LtxProjectFormatResult};

use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobResource, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::configs::ltx_roots::open_ltx_project;
use crate::plugins::configs::request::ConfigsFormatRequest;

/// Rewrite the LTX configs roots exposes.
///
/// Writing needs a file, so this refuses a project holding archived winners — the refusal comes from
/// `xrf-ltx` itself. Formatting an installation is therefore a legitimate refusal, not a gap.
///
/// Holds the roots exclusively for the whole run, so a second request over the same set is refused rather than allowed
/// to rewrite the files this one is walking. A cancelled run leaves the files it had already formatted formatted and
/// the rest untouched: each file is rewritten through a staged replace, so nothing is half-written and running it
/// again resolves the difference.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "format_directory"))]
#[tauri::command(rename = "format_directory")]
pub async fn configs_format_directory(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: ConfigsFormatRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<LtxProjectFormatResult> {
  let start: JobStart = JobStart::new(job_id, JobKind::ConfigsFormat).with_request(&request);

  let ConfigsFormatRequest { roots, prefix } = request;

  log::info!("Formatting ltx configs in {}", roots.describe());

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    start
      .with_exclusion_group(JobKind::ConfigsFormat.as_str())
      .with_resources(roots.roots.iter().map(|root| JobResource::tree(&root.path)).collect())
      .with_progress(progress),
  )?;

  // Off the async worker: this mounts every root, reads every config, and rewrites the ones that need it.
  run_job(
    &execution,
    "Configs formatting",
    registration,
    move || {
      let project: LtxProject = open_ltx_project(&roots, prefix.as_deref(), Default::default())?;

      project.format_all_files_opt(LtxFormatOptions::default().with_job(job))
    },
    |result| result.outcome,
  )
  .await
}
