use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_translation::{TranslationFormatOptions, TranslationFormatResult, TranslationFormatter};
use xrf_utils::{LineEndings, error_to_string};

use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobResource, JobStart, run_job};
use crate::core::types::TauriResult;
use crate::plugins::translations::request::TranslationsFormatRequest;
use crate::plugins::translations::state::TranslationProjectState;

/// Normalize the JSON translation sources under a directory.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "format_project"))]
#[tauri::command(rename = "format_project")]
pub async fn translations_format_project(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  project: State<'_, TranslationProjectState>,
  request: TranslationsFormatRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<TranslationFormatResult> {
  let start: JobStart = JobStart::new(job_id, JobKind::TranslationsFormat).with_request(&request);

  let TranslationsFormatRequest {
    directory,
    line_endings,
  } = request;

  log::info!("Formatting translation sources in {}", directory.display());

  // Before the job exists, because this is a refusal rather than a failed run: the editor's in-memory buffers are not
  // covered by any lease, and a save after this would put the pre-format content back.
  project.require_no_open_session_over(&directory)?;

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    start
      .with_exclusion_group(JobKind::TranslationsFormat.as_str())
      .with_resources(vec![JobResource::tree(&directory)])
      .with_progress(progress),
  )?;

  run_job(
    &execution,
    "Translations formatting",
    registration,
    move || run(job, directory, line_endings, false),
    |result| result.outcome,
  )
  .await
}

/// Walk and judge the sources off the async worker, which is where every blocking crate call belongs.
pub(super) fn run(
  job: JobHandle,
  directory: PathBuf,
  line_endings: Option<String>,
  is_check: bool,
) -> TauriResult<TranslationFormatResult> {
  let line_endings: Option<LineEndings> = line_endings
    .as_deref()
    .map(LineEndings::from_str)
    .transpose()
    .map_err(error_to_string)?;

  let paths: Vec<PathBuf> = vec![directory];
  let options: TranslationFormatOptions = TranslationFormatOptions::default()
    .with_job(job)
    .with_line_endings(line_endings);

  if is_check {
    TranslationFormatter::check_format_opt(&paths, options)
  } else {
    TranslationFormatter::format_opt(&paths, options)
  }
  .map_err(error_to_string)
}
