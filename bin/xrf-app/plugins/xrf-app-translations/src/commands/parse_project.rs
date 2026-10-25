use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobResource, JobStart};
use xrf_job::JobProgress;
use xrf_translation::{TranslationLanguage, TranslationParseOptions, TranslationParseResult, TranslationParser};
use xrf_utils::{error_to_string, format_path};

use crate::ipc::translations::TRANSLATIONS_JOB_PARSE;
use crate::request::TranslationsParseRequest;

/// Import one language's raw XML string tables into JSON sources, one import at a time.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "parse_project")]
pub async fn translations_parse_project(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: TranslationsParseRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<TranslationParseResult> {
  let start: JobStart = JobStart::new(job_id, TRANSLATIONS_JOB_PARSE)
    .with_request(&request)
    .with_kind_exclusion()
    .with_resources(vec![JobResource::tree(&request.output_dir)])
    .with_progress(progress);

  let TranslationsParseRequest {
    roots,
    language,
    prefix,
    output_dir,
    file,
    is_overwrite,
    is_dry_run,
  } = request;
  let language: TranslationLanguage = language.into_single().map_err(error_to_string)?;

  log::info!(
    "Parsing translations: {} root(s), '{language}', into {}{}",
    roots.roots.len(),
    format_path(&output_dir),
    if is_dry_run { " (dry run)" } else { "" }
  );

  let result: TranslationParseResult = registry
    .run(
      &execution,
      start,
      move |job| {
        TranslationParser::parse(&TranslationParseOptions {
          job,
          output: xrf_output::OutputOptions::default(),
          roots,
          prefix,
          language,
          output_dir,
          file,
          is_overwrite,
          is_dry_run,
          link_policy: Default::default(),
        })
      },
      |result| result.outcome,
    )
    .await?;

  log::info!(
    "Parsed {} translation file(s), {} created, {} updated, {} finding(s)",
    result.census.files_read,
    result.census.files_created,
    result.census.files_updated,
    result.get_findings().len()
  );

  Ok(result)
}
