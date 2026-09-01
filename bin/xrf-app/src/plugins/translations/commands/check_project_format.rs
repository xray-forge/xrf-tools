use std::path::PathBuf;
use std::sync::Arc;

use serde_json::json;
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_translation::TranslationFormatResult;

use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::translations::commands::format_project::run;
use crate::plugins::translations::lease::CHECK_FORMAT_JOB_KIND;

/// Report which JSON translation sources under a directory are not normalized.
///
/// Read-only, so no lease is taken and no open editor session is refused: two readers of one tree have nothing to
/// collide over, and a check that leaves every file exactly as it found it cannot make a buffer stale. A separate kind
/// from the rewrite it reports on, because they are different work with different consequences — one answers a
/// question, the other changes the files.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "check_project_format"))]
#[tauri::command(rename = "check_project_format")]
pub async fn translations_check_project_format(
  registry: State<'_, Arc<JobRegistry>>,
  directory: PathBuf,
  line_endings: Option<String>,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<TranslationFormatResult> {
  log::info!("Checking translation source format in {}", directory.display());

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, CHECK_FORMAT_JOB_KIND)
      .with_request(&json!({ "directory": directory, "lineEndings": line_endings }))
      .with_progress(progress),
  )?;

  let outcome: TauriResult<TranslationFormatResult> = run(job.clone(), directory, line_endings, true).await;

  registration.conclude_with(&outcome, job.is_cancelled());

  outcome
}
