use std::time::Instant;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

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

    // Stated before any result, because every count below is measured over what actually mounted: a source that failed
    // to open makes its assets look missing rather than unread.
    if !self.skipped_mounts().is_empty() {
      xrf_output::warning!(
        options.output,
        "{} declared source(s) could not be opened, so results below do not cover them:",
        self.skipped_mounts().len()
      );

      for skipped in self.skipped_mounts() {
        xrf_output::warning!(
          options.output,
          "  - {} at {}: {}",
          skipped.origin,
          format_path(&skipped.path),
          skipped.reason
        );
      }
    }

    xrf_output::info!(options.output, "");

    let started_at: Instant = Instant::now();
    let mut result: GamedataVerificationReport = GamedataVerificationReport::default();

    for check in checks {
      result.add_report(check.run(self, options));
    }

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
  use crate::{GamedataProjectVerifyOptions, GamedataVerificationStatus, GamedataVerificationType};

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

    assert_eq!(report.get_checks().len(), 1);
    assert_eq!(
      report.get_checks()[0].get_verification_type(),
      GamedataVerificationType::Levels
    );
    // The test project ships no spawn file, so the level roster is unknown and nothing is checked.
    assert_eq!(report.get_status(), GamedataVerificationStatus::Skipped);
  }
}
