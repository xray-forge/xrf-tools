use std::time::Duration;

use serde::Serialize;
use xrf_app_core::TauriResult;
use xrf_app_core::jobs::JobPhase;
use xrf_error::{XrfError, XrfResult};
use xrf_gamedata::{
  GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions, GamedataVerificationReport,
  GamedataVerificationType,
};
use xrf_job::{JobHandle, JobOutcome, JobScope};
use xrf_report::Status;
use xrf_utils::error_to_string;

use crate::GamedataCheckSummary;
use crate::request::GamedataVerifyRequest;

/// What a whole verification reports back to the desktop surface.
#[derive(Clone, Debug, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct GamedataVerifySummary {
  /// Whether every selected check finished, or the run stopped before or inside a check.
  pub outcome: JobOutcome,
  /// The aggregate verdict over the checks that ran.
  pub status: Status,
  pub checks: Vec<GamedataCheckSummary>,
  #[serde(with = "xrf_utils::duration_ms")]
  #[cfg_attr(feature = "typescript-bindings", specta(type = u64))]
  pub duration: Duration,
}

impl GamedataVerifySummary {
  /// Opens the project `request` names and runs its checks, every one this build knows where it names none, under
  /// `job`: a cancel taken before or inside a check answers a stopped run rather than an error.
  ///
  /// # Errors
  ///
  /// Returns why the project could not be opened or verified.
  pub fn run(request: GamedataVerifyRequest, job: &JobHandle) -> TauriResult<Self> {
    // todo: Return each check's findings (rule, subject, message), capped per check, so the verifier page can list
    //  them rather than their count alone.
    match Self::verify(request, job) {
      Ok(report) => Ok(Self::of(&report, job.elapsed())),
      Err(XrfError::Cancelled { .. }) => Ok(Self::cancelled(job.elapsed())),
      Err(error) => Err(error_to_string(error)),
    }
  }

  /// What a finished report says, as the desktop surface reads it.
  pub fn of(report: &GamedataVerificationReport, elapsed: Duration) -> Self {
    Self {
      outcome: report.get_outcome(),
      status: report.get_status(),
      checks: report.get_checks().iter().map(GamedataCheckSummary::of).collect(),
      duration: elapsed,
    }
  }

  /// A run stopped before it reported anything.
  pub fn cancelled(elapsed: Duration) -> Self {
    Self {
      outcome: JobOutcome::Cancelled,
      status: Status::Incomplete,
      checks: Vec::new(),
      duration: elapsed,
    }
  }

  fn verify(request: GamedataVerifyRequest, job: &JobHandle) -> XrfResult<GamedataVerificationReport> {
    job.check_cancelled()?;

    let project: GamedataProject = {
      let _opening: JobScope = job.enter(JobPhase::PREPARE, None);

      job.check_cancelled()?;

      GamedataProject::open(&GamedataProjectReadOptions {
        engine: request.engine,
        root: request.root,
        is_strict: request.is_strict,
        ..Default::default()
      })?
    };

    job.check_cancelled()?;

    project.verify(&GamedataProjectVerifyOptions {
      is_strict: request.is_strict,
      checks: request.checks.unwrap_or_else(GamedataVerificationType::get_all),
      job: job.clone(),
      ..Default::default()
    })
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;
  use std::sync::{Arc, Mutex};
  use std::time::Duration;

  use xrf_app_core::jobs::JobPhase;
  use xrf_gamedata::GamedataVerificationType;
  use xrf_job::{JobHandle, JobOutcome, JobProgress, ProgressSink};
  use xrf_report::Status;
  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use crate::GamedataVerifySummary;
  use crate::request::GamedataVerifyRequest;

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
      checks: Some(vec![GamedataVerificationType::Scripts]),
      engine: xrf_engine_target::XrayEngineChoice::Vanilla,
      is_strict: false,
      root,
    }
  }

  #[test]
  fn a_queued_cancel_does_not_open_the_project() {
    let job = JobHandle::inert();
    job.cancel();
    let summary = GamedataVerifySummary::run(request(PathBuf::new()), &job)
      .expect("cancellation is a result, not an invalid-root failure");

    assert_eq!(summary.outcome, JobOutcome::Cancelled);
    assert_eq!(summary.status, Status::Incomplete);
    assert!(summary.checks.is_empty());
  }

  #[test]
  fn cancelling_while_the_project_opens_skips_the_read() {
    let sink = Arc::new(CancelAtPhase {
      job: Mutex::new(None),
      phase: JobPhase::PREPARE,
      completed: None,
    });
    let job = JobHandle::new(sink.clone());
    *sink.job.lock().expect("test job lock") = Some(job.clone());

    let summary =
      GamedataVerifySummary::run(request(PathBuf::new()), &job).expect("cancellation skips the invalid root");

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

    let summary = GamedataVerifySummary::run(request(root), &job).expect("verification finishes");

    assert!(job.is_cancelled());
    assert_eq!(summary.outcome, JobOutcome::Completed);
    // Coverage and collisions always run first; the selected check finished after them.
    assert!(
      summary
        .checks
        .iter()
        .any(|check| check.check == GamedataVerificationType::Scripts)
    );
  }
}
