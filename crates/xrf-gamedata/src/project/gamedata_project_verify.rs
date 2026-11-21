use std::time::Instant;

use xrf_error::{XrfError, XrfResult};
use xrf_job::parallel::prelude::*;

use crate::project::events::GamedataEvent;
use crate::project::gamedata_check_schedule::GamedataCheckSchedule;
use crate::project::gamedata_project::GamedataProject;
use crate::project::gamedata_project_verify_options::GamedataProjectVerifyOptions;
use crate::project::gamedata_verification_check_report::GamedataVerificationCheckReport;
use crate::project::gamedata_verification_report::GamedataVerificationReport;
use crate::project::gamedata_verification_type::GamedataVerificationType;

/// How many checks may be working at once while somebody is reading.
const CHECK_WINDOW: usize = 6;

/// Phase a verification reports while it works through the checks it was asked for.
pub(crate) const PHASE_CHECKS: &str = "checks";

impl GamedataProject {
  pub fn verify(&self, options: &GamedataProjectVerifyOptions) -> XrfResult<GamedataVerificationReport> {
    let checks: Vec<GamedataVerificationType> = options.selected_checks();

    if checks.is_empty() {
      return Err(XrfError::new_unexpected_error("No gamedata checks to perform provided"));
    }

    options.emit(GamedataEvent::Started {
      root: self.root(),
      checks: &checks,
    });

    let started_at: Instant = Instant::now();
    let mut result: GamedataVerificationReport = GamedataVerificationReport::default();

    // First, and outside the progress scope below: neither coverage nor reachability is one of the kinds a caller
    // selects, and both are decided by what mounting already did rather than by work worth counting.
    for always_run in [GamedataVerificationType::Coverage, GamedataVerificationType::Collisions] {
      let report: GamedataVerificationCheckReport = always_run.run(self, options);

      options.emit(GamedataEvent::CheckFinished { report: &report });
      result.add_report(report);
    }

    // One level over the selected checks, advanced once per finished check and by nothing else. A check running alone
    // nests its own levels inside it, so a lone LTX check reads as `checks 0/1` above `verify 400/2000`; checks running
    // beside each other report on handles of their own that reach no sink, because one stack would show them as nested
    // in each other, and their per-file detail would replace the schedule's list of what is running.
    let verifying: xrf_job::JobScope = options.job.enter(PHASE_CHECKS, Some(checks.len() as u64));

    // Checks run together on whatever pool the caller installed, because most of them cannot use one on their own:
    // eight of the fifteen are strictly serial, and on Anomaly they account for 6.7 of the 15 seconds a sweep takes.
    // Overlapping them lets that time hide inside the checks that do fan out, rather than being added to them.
    //
    // What they find does not run together. Each wave's reports are handed out in selection order once the wave is
    // done, so a sweep says the same things in the same order however its workers were scheduled — which is the
    // contract `--jobs` states and what the reports are compared on.
    let schedule: GamedataCheckSchedule = GamedataCheckSchedule::default();

    // A run nobody listens to has no reader to hand reports to, so it pays nothing for one: the window buys
    // responsiveness, and where nobody is watching that is not worth a slower sweep. A scripted run is this case.
    let window: usize = if options.listener.is_some() {
      CHECK_WINDOW
    } else {
      checks.len()
    };

    // Waves rather than all at once, and the outer loop is intentionally sequential: a wave finishes before the next
    // starts, so its reports are handed out together and a run says something every second or so instead of saying
    // nothing and then everything.
    let mut reports: Vec<Option<GamedataVerificationCheckReport>> = Vec::with_capacity(checks.len());

    for (wave, released_together) in checks.chunks(window).enumerate() {
      let is_alone: bool = released_together.len() == 1;
      let mut released: Vec<Option<GamedataVerificationCheckReport>> = released_together
        .par_iter()
        .enumerate()
        .map(|(offset, check)| {
          let index: usize = wave * window + offset;

          // Do not start another check after cancellation. Running checks stop at their own iteration boundaries.
          if options.job.is_cancelled() {
            return None;
          }

          schedule.enter(&options.job, index, *check);

          let report: GamedataVerificationCheckReport = check.run(
            self,
            &GamedataProjectVerifyOptions {
              job: if is_alone {
                options.job.clone()
              } else {
                options.job.without_progress()
              },
              ..options.clone()
            },
          );

          schedule.leave(&options.job, index);
          verifying.advance();

          Some(report)
        })
        .collect();

      for report in released.iter().flatten() {
        options.emit(GamedataEvent::CheckFinished { report });
      }

      reports.append(&mut released);
    }

    // In selection order whatever order they finished in, so the report describes the request rather than the schedule.
    if reports.iter().any(Option::is_none) {
      result.set_outcome(xrf_job::JobOutcome::Cancelled);
    }

    for report in reports.into_iter().flatten() {
      result.add_report(report);
    }

    options.job.set_detail(None);

    result.set_duration(started_at.elapsed());

    Ok(result)
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;
  use std::sync::{Arc, Mutex};

  use xrf_ltx_project::LtxProject;
  use xrf_vfs::{XraySearch, XrayVfs};

  use super::GamedataProject;
  use crate::project::events::GamedataEvent;
  use crate::project::gamedata_project_verify_options::GamedataProjectVerifyOptions;
  use crate::project::gamedata_verification_check_report::GamedataVerificationCheckReport;
  use crate::project::gamedata_verification_status::GamedataVerificationStatus;
  use crate::project::gamedata_verification_type::GamedataVerificationType;

  /// A project with nothing mounted, for asserting which checks run rather than what they find.
  fn empty_project() -> GamedataProject {
    GamedataProject {
      engine: xrf_engine_target::XrayEngineResolution::named(xrf_engine_target::XrayEngine::Vanilla),
      ltx_project: LtxProject::empty(PathBuf::new()),
      root: PathBuf::new(),
      search: XraySearch::new(XrayVfs::new()),
    }
  }

  #[test]
  fn every_check_accepts_cancellation_before_reading_its_inputs() {
    let project = empty_project();
    let options = GamedataProjectVerifyOptions::default();
    options.job.cancel();

    for check in GamedataVerificationType::ALL {
      let report = check.run(&project, &options);

      assert_eq!(report.get_status(), GamedataVerificationStatus::Incomplete, "{check}");
      assert!(
        report.get_findings().is_empty(),
        "{check} read missing inputs despite cancellation"
      );
    }
  }

  /// What a listener hears is the run's start, then each check as it finishes, in the order of the report.
  #[test]
  fn hands_out_each_finished_check_in_report_order() {
    let project: GamedataProject = empty_project();
    let heard: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let recorded: Arc<Mutex<Vec<String>>> = Arc::clone(&heard);
    let options: GamedataProjectVerifyOptions = GamedataProjectVerifyOptions {
      checks: vec![GamedataVerificationType::Levels, GamedataVerificationType::Spawns],
      listener: Some(Arc::new(move |event: &GamedataEvent<'_>| {
        recorded.lock().expect("record").push(match event {
          GamedataEvent::Started { checks, .. } => format!("started {}", checks.len()),
          GamedataEvent::CheckFinished { report } => report.get_verification_type().to_string(),
        });
      })),
      ..Default::default()
    };

    let report = project.verify(&options).expect("Expected verification to complete");
    let reported: Vec<String> = report
      .get_checks()
      .iter()
      .map(|check| check.get_verification_type().to_string())
      .collect();

    assert_eq!(heard.lock().expect("heard")[0], "started 2");
    assert_eq!(heard.lock().expect("heard")[1..], reported);
  }

  #[test]
  fn runs_each_selected_check_once_in_request_order() {
    let project: GamedataProject = empty_project();
    let options: GamedataProjectVerifyOptions = GamedataProjectVerifyOptions {
      checks: vec![GamedataVerificationType::Levels, GamedataVerificationType::Levels],
      ..Default::default()
    };

    let report = project
      .verify(&options)
      .expect("Expected level verification to complete");

    // Both always-run checks, then the selection deduplicated to one entry.
    assert_eq!(
      report
        .get_checks()
        .iter()
        .map(GamedataVerificationCheckReport::get_verification_type)
        .collect::<Vec<_>>(),
      vec![
        GamedataVerificationType::Coverage,
        GamedataVerificationType::Collisions,
        GamedataVerificationType::Levels,
      ]
    );
    // The test project ships no spawn file, so the level roster is unknown and nothing is checked; nothing collides and
    // nothing was skipped either, so neither always-run check touches that verdict.
    assert_eq!(report.get_status(), GamedataVerificationStatus::Skipped);
  }
}
