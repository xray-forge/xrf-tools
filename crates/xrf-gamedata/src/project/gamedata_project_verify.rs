use std::time::Instant;

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use xrf_error::{XrfError, XrfResult};
use xrf_output::{OutputSequence, OutputSlot};
use xrf_utils::format_path;

use crate::project::gamedata_check_schedule::GamedataCheckSchedule;
use crate::project::job_phases::GAMEDATA_PHASE_CHECKS;
use crate::{
  GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationCheckReport, GamedataVerificationReport,
  GamedataVerificationType,
};

impl GamedataProject {
  pub fn verify(&self, options: &GamedataProjectVerifyOptions) -> XrfResult<GamedataVerificationReport> {
    let checks: Vec<GamedataVerificationType> = options.selected_checks();

    if checks.is_empty() {
      return Err(XrfError::new_unexpected_error("No gamedata checks to perform provided"));
    }

    xrf_output::info!(
      options.output,
      "Verifying gamedata project: {}",
      format_path(self.root())
    );

    xrf_output::info!(
      options.output,
      "Verifying modules: \n  -{}",
      checks.iter().map(ToString::to_string).collect::<Vec<_>>().join("\n  -")
    );

    xrf_output::info!(options.output, "");

    let started_at: Instant = Instant::now();
    let mut result: GamedataVerificationReport = GamedataVerificationReport::default();

    // First, and outside the progress scope below: neither coverage nor reachability is one of the kinds a caller
    // selects, and both are decided by what mounting already did rather than by work worth counting.
    result.add_report(GamedataVerificationType::Coverage.run(self, options));
    result.add_report(GamedataVerificationType::Collisions.run(self, options));

    // One level over the selected checks, and each check that reports its own work nests inside it: the LTX checks
    // enter their own file-level scopes on this same handle, so a run reads as `checks 3/11` above `verify 400/2000`
    // rather than as one bar that sits still for minutes. This is the first consumer to nest across two crates.
    let verifying: xrf_job::JobScope = options.job.enter(GAMEDATA_PHASE_CHECKS, Some(checks.len() as u64));

    // Checks run together on whatever pool the caller installed, because most of them cannot use one on their own:
    // eight of the fifteen are strictly serial, and on Anomaly they account for 6.7 of the 15 seconds a sweep takes.
    // Overlapping them lets that time hide inside the checks that do fan out, rather than being added to them.
    //
    // Their output does not run together. Each check writes into its listed position and the sequence releases those
    // positions in selection order, so a sweep says the same things in the same order however its workers were
    // scheduled — which is the contract `--jobs` states and what the reports are compared on.
    let sequence: OutputSequence = OutputSequence::new(&options.output, checks.len());
    let schedule: GamedataCheckSchedule = GamedataCheckSchedule::default();

    let reports: Vec<Option<GamedataVerificationCheckReport>> = checks
      .par_iter()
      .enumerate()
      .map(|(index, check)| {
        // Before a check starts, never inside one. A check already running is left to finish: they parallelise
        // internally and have no boundary of their own, so the only safe place to stop is where one has not begun.
        if options.job.is_cancelled() {
          return None;
        }

        let slot: OutputSlot = sequence.new_slot(index);

        schedule.enter(&options.job, index, *check);

        let report: GamedataVerificationCheckReport = check.run(
          self,
          &GamedataProjectVerifyOptions {
            output: slot.get_output().clone(),
            job: options.job.clone(),
            is_strict: options.is_strict,
            checks: Vec::new(),
          },
        );

        schedule.leave(&options.job, index);
        verifying.advance();

        Some(report)
      })
      .collect();

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

  use xrf_ltx::LtxProject;
  use xrf_vfs::XrayLookupScope;

  use super::GamedataProject;
  use crate::{
    GamedataProjectVerifyOptions, GamedataVerificationCheckReport, GamedataVerificationStatus, GamedataVerificationType,
  };

  /// A project with nothing mounted, for asserting which checks run rather than what they find.
  fn empty_project() -> GamedataProject {
    GamedataProject {
      ltx_project: LtxProject::empty(PathBuf::new()),
      root: PathBuf::new(),
      scope: XrayLookupScope::all(),
    }
  }

  #[test]
  fn runs_each_selected_check_once_in_request_order() {
    let project: GamedataProject = empty_project();
    let options: GamedataProjectVerifyOptions = GamedataProjectVerifyOptions {
      checks: vec![GamedataVerificationType::Levels, GamedataVerificationType::Levels],
      output: xrf_output::OutputOptions::default(),
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
