use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use serde_json::json;
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_translation::{ProjectFormatOptions, ProjectFormatResult, format_sources};
use xrf_utils::LineEndings;

use crate::core::error::error_to_string;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::translations::lease::{FORMAT_JOB_KIND, to_output_lease_key};
use crate::plugins::translations::state::TranslationProjectState;

/// Normalize the JSON translation sources under a directory.
///
/// A host directory rather than mounted roots, because this rewrites its sources in place and there is nowhere to
/// put a file inside an archive volume. `configs format_directory` takes roots because an LTX project is a VFS
/// notion — winning configs, an include graph, archived entries it declines — and none of that applies to flat JSON.
///
/// Holds the directory exclusively for the whole run, under the same lease a build and an import take, so a second
/// writer over the same tree is refused rather than allowed to read files this one is midway through replacing. A
/// cancelled run leaves the sources it had already formatted formatted and the rest untouched: each is rewritten
/// through a staged replace, so nothing is half-written and running it again resolves the difference.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "format_project"))]
#[tauri::command(rename = "format_project")]
pub async fn translations_format_project(
  registry: State<'_, Arc<JobRegistry>>,
  project: State<'_, TranslationProjectState>,
  directory: PathBuf,
  line_endings: Option<String>,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<ProjectFormatResult> {
  log::info!("Formatting translation sources in {}", directory.display());

  // Before the job exists, because this is a refusal rather than a failed run: the editor's in-memory buffers are not
  // covered by any lease, and a save after this would put the pre-format content back.
  project.require_no_open_session_over(&directory)?;

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, FORMAT_JOB_KIND)
      .with_lease_keys(vec![to_output_lease_key(&directory)])
      .with_request(&json!({ "directory": directory, "lineEndings": line_endings }))
      .with_progress(progress),
  )?;

  let outcome: TauriResult<ProjectFormatResult> = run(job.clone(), directory, line_endings, false).await;

  registration.conclude_with(&outcome, job.is_cancelled());

  outcome
}

/// Walk and judge the sources off the async worker, which is where every blocking crate call belongs.
pub(super) async fn run(
  job: JobHandle,
  directory: PathBuf,
  line_endings: Option<String>,
  is_check: bool,
) -> TauriResult<ProjectFormatResult> {
  let line_endings: Option<LineEndings> = line_endings
    .as_deref()
    .map(LineEndings::from_str)
    .transpose()
    .map_err(error_to_string)?;

  tauri::async_runtime::spawn_blocking(move || {
    format_sources(&ProjectFormatOptions {
      job,
      output: Default::default(),
      paths: vec![directory],
      is_check,
      line_endings,
    })
  })
  .await
  .map_err(|error| format!("Translations formatting did not finish: {error}"))?
  .map_err(error_to_string)
}
