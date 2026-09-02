use std::sync::Arc;

use serde_json::json;
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_ltx::{LtxProject, LtxProjectOptions, LtxProjectVerifyResult, LtxVerifyOptions};
use xrf_vfs::XrayRoots;

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::configs::lease::VERIFY_JOB_KIND;
use crate::plugins::configs::ltx_roots::open_ltx_project;

/// Verify the LTX configs roots exposes.
///
/// Read-only, so it goes through the roots and covers archived configs too. `xrf-ltx` draws the same
/// line: its read-only check reads through the VFS where its rewrite refuses archived winners.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "verify_directory"))]
#[tauri::command(rename = "verify_directory")]
pub async fn configs_verify_directory(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  roots: XrayRoots,
  prefix: Option<String>,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<LtxProjectVerifyResult> {
  log::info!("Verifying ltx configs in {}", roots.describe());

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, VERIFY_JOB_KIND)
      .with_request(&json!({ "roots": roots, "prefix": prefix }))
      .with_progress(progress),
  )?;

  // Off the async worker: opening the project mounts every root and reads every config it holds, and the check then
  // walks all of them. An `async fn` alone would leave that on an executor thread meant for short requests.
  let verifying: JobHandle = job.clone();
  let outcome: TauriResult<LtxProjectVerifyResult> = execution
    .run_blocking("Configs verification", move || {
      let project: LtxProject = open_ltx_project(
        &roots,
        prefix.as_deref(),
        LtxProjectOptions {
          is_with_schemes_check: true,
          // todo: Probably should be provided as parameter.
          is_strict_check: false,
        },
      )?;

      project.verify_entries_opt(LtxVerifyOptions::default().with_job(verifying))
    })
    .await?
    .map_err(error_to_string);

  registration.conclude_with(&outcome, job.is_cancelled());

  outcome
}
