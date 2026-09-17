use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_dltx::select_ltx_dialect;
use xrf_job::{JobHandle, JobProgress, JobScope};
use xrf_ltx::{LtxProject, LtxProjectOptions, LtxProjectVerifyResult, LtxVerifyOptions};

use crate::core::execution::ExecutionState;
use crate::core::jobs::{JOB_PHASE_PREPARE, JobKind, JobRegistration, JobRegistry, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::configs::request::ConfigsVerifyRequest;

/// Verifies LTX configs through the VFS, including archived files.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "verify_directory"))]
#[tauri::command(rename = "verify_directory")]
pub async fn configs_verify_directory(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: ConfigsVerifyRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<LtxProjectVerifyResult> {
  let start: JobStart = JobStart::new(job_id, JobKind::ConfigsVerify).with_request(&request);

  let ConfigsVerifyRequest { roots, prefix, is_dltx } = request;

  log::info!("Verifying ltx configs in {}", roots.describe());

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    start
      .with_exclusion_group(JobKind::ConfigsVerify.as_str())
      .with_progress(progress),
  )?;

  run_job(
    &execution,
    "Configs verification",
    registration,
    move || {
      let project: LtxProject = {
        let _preparing: JobScope = job.enter(JOB_PHASE_PREPARE, None);

        LtxProject::open_at_roots_opt(
          &roots,
          prefix.as_deref(),
          LtxProjectOptions {
            // One sweep over a directory, then the project is dropped; nothing asks twice.
            is_caching_resolutions: false,
            dialect: select_ltx_dialect(is_dltx),
            is_with_schemes_check: true,
            // todo: Probably should be provided as parameter.
            is_strict_check: false,
          },
        )?
      };

      project.verify_entries_opt(LtxVerifyOptions::default().with_job(job))
    },
    |result| result.outcome,
  )
  .await
}
