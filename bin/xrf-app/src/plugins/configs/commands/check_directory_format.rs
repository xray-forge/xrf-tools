use std::sync::Arc;

use serde_json::json;
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_ltx::{LtxFormatOptions, LtxProject, LtxProjectFormatResult};

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::configs::lease::CHECK_FORMAT_JOB_KIND;
use crate::plugins::configs::ltx_roots::open_ltx_project;
use crate::plugins::configs::request::ConfigsFormatRequest;

/// Report which LTX configs roots exposes are misformatted.
///
/// Read-only, so an archived config is checked like any other, and no lease is taken: two readers of one project have
/// nothing to collide over. A separate kind from the rewrite it reports on, because they are different work with
/// different consequences — one answers a question, the other changes the files.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "check_directory_format"))]
#[tauri::command(rename = "check_directory_format")]
pub async fn configs_check_directory_format(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: ConfigsFormatRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<LtxProjectFormatResult> {
  let ConfigsFormatRequest { roots, prefix } = request;

  log::info!("Checking ltx format in {}", roots.describe());

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, CHECK_FORMAT_JOB_KIND)
      .with_request(&json!({ "roots": roots, "prefix": prefix }))
      .with_progress(progress),
  )?;

  // Off the async worker: this mounts every root and reads every config it holds.
  let checking: JobHandle = job.clone();
  let outcome: TauriResult<LtxProjectFormatResult> = execution
    .run_blocking("Configs format check", move || {
      let project: LtxProject = open_ltx_project(&roots, prefix.as_deref(), Default::default())?;

      project.check_format_all_files_opt(LtxFormatOptions::default().with_job(checking))
    })
    .await?
    .map_err(error_to_string);

  registration.conclude_with(&outcome, job.is_cancelled());

  outcome
}
