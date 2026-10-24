use std::sync::Arc;

use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::{JobRegistry, JobResource, JobStart};
use xrf_job::JobProgress;
use xrf_translation::{TranslationBuildOptions, TranslationBuildResult, TranslationBuilder};
use xrf_utils::format_path;

use crate::ipc::translations::TRANSLATIONS_JOB_BUILD;
use crate::request::TranslationsBuildRequest;

/// Compile translation sources into per-language string tables, one build at a time.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "build_project"))]
#[tauri::command(rename = "build_project")]
pub async fn translations_build_project(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: TranslationsBuildRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<TranslationBuildResult> {
  let start: JobStart = JobStart::new(job_id, TRANSLATIONS_JOB_BUILD)
    .with_request(&request)
    .with_kind_exclusion()
    .with_resources(vec![JobResource::tree(&request.output_dir)])
    .with_progress(progress);

  // `all` is accepted: compiling every language at once is the ordinary build.
  let TranslationsBuildRequest {
    roots,
    prefix,
    language,
    output_dir,
    is_sorted,
  } = request;

  log::info!(
    "Building translations: {} root(s), '{language}', into {}",
    roots.roots.len(),
    format_path(&output_dir)
  );

  let result: TranslationBuildResult = registry
    .run(
      &execution,
      start,
      move |job| {
        TranslationBuilder::build_roots(
          &roots,
          prefix.as_deref(),
          &TranslationBuildOptions {
            job,
            is_sorted,
            output: xrf_output::OutputOptions::default(),
            output_dir,
            language,
            link_policy: Default::default(),
          },
        )
      },
      |result| result.outcome,
    )
    .await?;

  log::info!(
    "Built {} string table(s) from {} source(s)",
    result.files,
    result.sources
  );

  Ok(result)
}
