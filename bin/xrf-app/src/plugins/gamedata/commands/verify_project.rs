use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_gamedata::{
  GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions, GamedataVerificationCheckReport,
  GamedataVerificationReport, GamedataVerificationType,
};
use xrf_job::{JobHandle, JobOutcome, JobProgress};
use xrf_utils::format_path;

use crate::core::error::error_to_string;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};
use crate::core::types::TauriResult;
use crate::plugins::gamedata::lease::VERIFY_JOB_KIND;

/// One check's verdict, as the desktop surface shows it.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct GamedataCheckSummary {
  /// The check that ran, spelled as the command line spells it.
  pub check: String,
  /// `passed`, `failed`, `incomplete`, or `skipped`.
  pub status: String,
  /// The check's own one-line verdict.
  pub summary: String,
  pub findings: usize,
  /// How long this check took, where it measured itself.
  #[serde(with = "xrf_utils::optional_duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = Option<u64>))]
  pub duration: Option<Duration>,
}

/// What a whole verification reports back to the desktop surface.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct GamedataVerifySummary {
  /// Whether every selected check ran, or the run was stopped between them.
  ///
  /// A stopped run's checks are real verdicts; its silence about the rest is not one.
  pub outcome: JobOutcome,
  /// The aggregate verdict over the checks that ran.
  pub status: String,
  pub checks: Vec<GamedataCheckSummary>,
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
}

/// What a verification was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct GamedataVerifyRequest {
  /// Gamedata root to verify.
  pub root: PathBuf,
  /// Checks to run, or nothing for every one this build knows.
  pub checks: Option<Vec<String>>,
  /// Whether a check that would warn should fail instead.
  pub is_strict: bool,
}

/// Run the selected checks over a gamedata project.
///
/// Takes no lease: verification only reads, so two runs over one project have nothing to collide over. It is still a
/// job, because a full run over an installation is minutes of work that somebody may want to watch or call off.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "verify_project"))]
#[tauri::command(rename = "verify_project")]
pub async fn gamedata_verify_project(
  registry: State<'_, Arc<JobRegistry>>,
  request: GamedataVerifyRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<GamedataVerifySummary> {
  log::info!("Verifying gamedata project: {}", format_path(&request.root));

  let checks: Vec<GamedataVerificationType> = match &request.checks {
    Some(names) => names
      .iter()
      .map(|name| name.parse::<GamedataVerificationType>().map_err(error_to_string))
      .collect::<TauriResult<Vec<GamedataVerificationType>>>()?,
    None => GamedataVerificationType::get_all(),
  };

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    JobStart::new(job_id, VERIFY_JOB_KIND)
      .with_request(&json!({ "root": request.root, "checks": request.checks }))
      .with_progress(progress),
  )?;

  // Off the async worker: this mounts an installation, indexes every asset it declares, and runs checks that
  // parallelise internally. None of that belongs on an executor thread meant for short requests.
  let verifying: JobHandle = job.clone();
  let is_strict: bool = request.is_strict;
  let root: PathBuf = request.root;

  let outcome: TauriResult<GamedataVerifySummary> = tauri::async_runtime::spawn_blocking(move || {
    let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
      root,
      is_strict,
      ..Default::default()
    })?;

    project.verify(&GamedataProjectVerifyOptions {
      is_strict,
      checks,
      job: verifying.clone(),
      ..Default::default()
    })
  })
  .await
  .map_err(|error| format!("Gamedata verification did not finish: {error}"))?
  .map_err(error_to_string)
  .map(|report: GamedataVerificationReport| to_summary(&report, job.elapsed()));

  registration.conclude_with(&outcome, job.is_cancelled());

  // todo: Better reporting and display so findings actually can be analysed from UI.
  outcome
}

/// The report as the desktop surface reads it.
fn to_summary(report: &GamedataVerificationReport, elapsed: Duration) -> GamedataVerifySummary {
  GamedataVerifySummary {
    outcome: report.get_outcome(),
    status: report.get_status().to_string(),
    checks: report
      .get_checks()
      .iter()
      .map(|check: &GamedataVerificationCheckReport| GamedataCheckSummary {
        check: check.get_verification_type().to_string(),
        status: check.get_status().to_string(),
        summary: check.get_summary().to_owned(),
        findings: check.get_findings().len(),
        duration: check.get_duration(),
      })
      .collect(),
    // The job's own clock rather than the report's, so opening the project is inside the number a person reads.
    duration: elapsed,
  }
}
