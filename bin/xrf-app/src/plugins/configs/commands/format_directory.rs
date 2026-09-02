use std::sync::Arc;

use serde_json::json;
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_ltx::{LtxFormatOptions, LtxProject, LtxProjectFormatResult};
use xrf_vfs::XrayRoots;

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::configs::lease::{FORMAT_JOB_KIND, to_format_lease_key};
use crate::plugins::configs::ltx_roots::open_ltx_project;

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
  roots: XrayRoots,
  prefix: Option<String>,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<LtxProjectFormatResult> {
  log::info!("Formatting ltx configs in {}", roots.describe());

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, FORMAT_JOB_KIND)
      .with_lease_keys(vec![to_format_lease_key(&roots, prefix.as_deref())])
      .with_request(&json!({ "roots": roots, "prefix": prefix }))
      .with_progress(progress),
  )?;

  // Off the async worker: this mounts every root, reads every config, and rewrites the ones that need it.
  let formatting: JobHandle = job.clone();
  let outcome: TauriResult<LtxProjectFormatResult> = execution
    .run_blocking("Configs formatting", move || {
      let project: LtxProject = open_ltx_project(&roots, prefix.as_deref(), Default::default())?;

      project.format_all_files_opt(LtxFormatOptions::default().with_job(formatting))
    })
    .await?
    .map_err(error_to_string);

  registration.conclude_with(&outcome, job.is_cancelled());

  outcome
}
