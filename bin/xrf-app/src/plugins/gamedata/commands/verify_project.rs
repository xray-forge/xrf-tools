use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::State;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_error::{XrfError, XrfResult};
use xrf_gamedata::{
  GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions, GamedataVerificationCheckReport,
  GamedataVerificationReport, GamedataVerificationType,
};
use xrf_job::{JobHandle, JobOutcome, JobProgress, JobScope};
use xrf_utils::format_path;

use crate::core::error::error_to_string;
use crate::core::execution::ExecutionState;
use crate::core::jobs::{JobKind, JobRegistration, JobRegistry, JobStart, run_job};
use crate::core::types::TauriResult;

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
  /// Whether every selected check finished, or the run stopped before or inside a check.
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
/// Holds the verification action group across windows. A full run over an installation is minutes of work that
/// somebody may want to watch or call off.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "verify_project"))]
#[tauri::command(rename = "verify_project")]
pub async fn gamedata_verify_project(
  execution: State<'_, ExecutionState>,
  registry: State<'_, Arc<JobRegistry>>,
  request: GamedataVerifyRequest,
  job_id: Uuid,
  progress: Channel<JobProgress>,
) -> TauriResult<GamedataVerifySummary> {
  let start: JobStart = JobStart::new(job_id, JobKind::GamedataVerify).with_request(&request);

  log::info!("Verifying gamedata project: {}", format_path(&request.root));

  let checks: Vec<GamedataVerificationType> = match &request.checks {
    Some(names) => names
      .iter()
      .map(|name| name.parse::<GamedataVerificationType>().map_err(error_to_string))
      .collect::<TauriResult<Vec<GamedataVerificationType>>>()?,
    None => GamedataVerificationType::get_all(),
  };

  let (job, registration): (JobHandle, JobRegistration) = registry.register(
    start
      .with_exclusion_group(JobKind::GamedataVerify.as_str())
      .with_progress(progress),
  )?;

  // Off the async worker: this mounts an installation, indexes every asset it declares, and runs checks that
  // parallelise internally. None of that belongs on an executor thread meant for short requests.
  run_job(
    &execution,
    "Gamedata verification",
    registration,
    move || run_verification(request, checks, &job),
    |summary| summary.outcome,
  )
  .await
}

/// Opens the project and verifies it, translating cooperative cancellation into a stopped result.
fn run_verification(
  request: GamedataVerifyRequest,
  checks: Vec<GamedataVerificationType>,
  job: &JobHandle,
) -> TauriResult<GamedataVerifySummary> {
  let report: XrfResult<GamedataVerificationReport> = (|| {
    job.check_cancelled()?;

    let project: GamedataProject = {
      let _opening: JobScope = job.enter("open", None);
      job.check_cancelled()?;

      GamedataProject::open(&GamedataProjectReadOptions {
        root: request.root,
        is_strict: request.is_strict,
        ..Default::default()
      })?
    };

    job.check_cancelled()?;

    project.verify(&GamedataProjectVerifyOptions {
      is_strict: request.is_strict,
      checks,
      job: job.clone(),
      ..Default::default()
    })
  })();

  // todo: Better reporting and display so findings actually can be analysed from UI.
  match report {
    Ok(report) => Ok(to_summary(&report, job.elapsed())),
    Err(XrfError::Cancelled { .. }) => Ok(GamedataVerifySummary {
      outcome: JobOutcome::Cancelled,
      status: String::from("incomplete"),
      checks: Vec::new(),
      duration: job.elapsed(),
    }),
    Err(error) => Err(error_to_string(error)),
  }
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

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;
  use std::sync::{Arc, Mutex};
  use std::time::Duration;

  use xrf_gamedata::GamedataVerificationType;
  use xrf_job::{JobHandle, JobOutcome, JobProgress, ProgressSink};
  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use super::{GamedataVerifyRequest, run_verification};

  struct CancelAtPhase {
    job: Mutex<Option<JobHandle>>,
    phase: &'static str,
    completed: Option<u64>,
  }

  impl ProgressSink for CancelAtPhase {
    fn report(&self, progress: &JobProgress) {
      if progress
        .levels
        .iter()
        .any(|level| level.id == self.phase && self.completed.is_none_or(|count| level.completed == count))
        && let Some(job) = self.job.lock().expect("test job lock").take()
      {
        job.cancel();
      }
    }
  }

  fn request(root: PathBuf) -> GamedataVerifyRequest {
    GamedataVerifyRequest {
      root,
      checks: None,
      is_strict: false,
    }
  }

  #[test]
  fn a_queued_cancel_does_not_open_the_project() {
    let job = JobHandle::inert();
    job.cancel();
    let summary = run_verification(request(PathBuf::new()), vec![GamedataVerificationType::Scripts], &job)
      .expect("cancellation is a result, not an invalid-root failure");

    assert_eq!(summary.outcome, JobOutcome::Cancelled);
    assert_eq!(summary.status, "incomplete");
    assert!(summary.checks.is_empty());
  }

  #[test]
  fn cancelling_at_the_open_phase_skips_the_read() {
    let sink = Arc::new(CancelAtPhase {
      job: Mutex::new(None),
      phase: "open",
      completed: None,
    });
    let job = JobHandle::new(sink.clone());
    *sink.job.lock().expect("test job lock") = Some(job.clone());

    let summary = run_verification(request(PathBuf::new()), vec![GamedataVerificationType::Scripts], &job)
      .expect("cancellation skips the invalid root");

    assert_eq!(summary.outcome, JobOutcome::Cancelled);
    assert!(summary.checks.is_empty());
  }

  #[test]
  fn a_cancel_after_the_last_check_keeps_the_completed_result() {
    let root = build_absolute_generated_test_resource_path("gamedata_job/late_cancel");
    fs::create_dir_all(root.join("configs")).expect("configs directory");
    fs::write(root.join("configs/system.ltx"), "[system]\nversion = 1\n").expect("system config");
    let sink = Arc::new(CancelAtPhase {
      job: Mutex::new(None),
      phase: "checks",
      completed: Some(1),
    });
    // Only phase transitions report: the final snapshot comes as the completed check scope is dropped.
    let job = JobHandle::with_interval(sink.clone(), Duration::MAX);
    *sink.job.lock().expect("test job lock") = Some(job.clone());

    let summary =
      run_verification(request(root), vec![GamedataVerificationType::Scripts], &job).expect("verification finishes");

    assert!(job.is_cancelled());
    assert_eq!(summary.outcome, JobOutcome::Completed);
  }
}
