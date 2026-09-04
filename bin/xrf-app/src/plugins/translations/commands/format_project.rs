use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use serde_json::json;
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress};
use xrf_translation::{TranslationFormatOptions, TranslationFormatResult, TranslationFormatter};
use xrf_utils::LineEndings;

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::translations::lease::{FORMAT_JOB_KIND, to_output_lease_key};
use crate::plugins::translations::request::TranslationsFormatRequest;
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
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  project: State<'_, TranslationProjectState>,
  request: TranslationsFormatRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<TranslationFormatResult> {
  let TranslationsFormatRequest {
    directory,
    line_endings,
  } = request;

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

  let outcome: TauriResult<TranslationFormatResult> =
    run(&execution, job.clone(), directory, line_endings, false).await;

  registration.conclude_with(&outcome, job.is_cancelled());

  outcome
}

/// Walk and judge the sources off the async worker, which is where every blocking crate call belongs.
///
/// `is_check` picks which of the formatter's two doors is opened, rather than being handed to one door that decides
/// for itself: whether this call rewrites the tree is the difference between the two commands above.
pub(super) async fn run(
  execution: &ExecutionState,
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

  execution
    .run_blocking("Translations formatting", move || {
      let paths: Vec<PathBuf> = vec![directory];
      let options: TranslationFormatOptions = TranslationFormatOptions::default()
        .with_job(job)
        .with_line_endings(line_endings);

      if is_check {
        TranslationFormatter::check_format_opt(&paths, options)
      } else {
        TranslationFormatter::format_opt(&paths, options)
      }
    })
    .await?
    .map_err(error_to_string)
}
