use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobPhase, JobRegistry, JobStart};
use xrf_dltx::DltxDialect;
use xrf_job::{JobProgress, JobScope};
use xrf_ltx::{LtxProject, LtxProjectOptions, LtxProjectVerifyResult, LtxVerifyOptions};

use crate::ipc::configs::CONFIGS_JOB_VERIFY;
use crate::request::ConfigsVerifyRequest;

/// Verifies LTX configs through the VFS, including archived files, one verification at a time.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "verify_directory"))]
#[tauri::command(rename = "verify_directory")]
pub async fn configs_verify_directory(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: ConfigsVerifyRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<LtxProjectVerifyResult> {
  let start: JobStart = JobStart::new(job_id, CONFIGS_JOB_VERIFY)
    .with_request(&request)
    .with_kind_exclusion()
    .with_progress(progress);

  let ConfigsVerifyRequest {
    roots,
    prefix,
    is_dltx,
    is_strict,
  } = request;

  log::info!("Verifying ltx configs in {}", roots.describe());

  registry
    .run(
      &execution,
      start,
      move |job| {
        let project: LtxProject = {
          let _preparing: JobScope = job.enter(JobPhase::PREPARE, None);

          LtxProject::open_at_roots_opt(
            &roots,
            prefix.as_deref(),
            LtxProjectOptions {
              // One sweep over a directory, then the project is dropped; nothing asks twice.
              is_caching_resolutions: false,
              dialect: DltxDialect::select(is_dltx),
              is_with_schemes_check: true,
              is_strict_check: is_strict,
            },
          )?
        };

        project.verify_entries_opt(LtxVerifyOptions::default().with_job(job))
      },
      |result| result.outcome,
    )
    .await
}
