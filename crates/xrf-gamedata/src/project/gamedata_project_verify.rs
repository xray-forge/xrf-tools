use std::time::Instant;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::project::job_phases::GAMEDATA_PHASE_CHECKS;
use crate::{GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationReport, GamedataVerificationType};

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

    for check in checks {
      // Between checks. A check already running is left to finish: they parallelise internally and have no boundary of
      // their own, so the only safe place to stop is where one ends and the next has not begun.
      if options.job.is_cancelled() {
        result.set_outcome(xrf_job::JobOutcome::Cancelled);

        break;
      }

      // Sequential here, so naming the check being run is meaningful.
      options.job.set_detail(Some(check.to_string()));

      result.add_report(check.run(self, options));
      verifying.advance();
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
