use std::path::Path;

use crate::project::gamedata_verification_check_report::GamedataVerificationCheckReport;
use crate::project::gamedata_verification_type::GamedataVerificationType;

/// What a gamedata verification tells its listener: that it started, and each check as it finishes.
///
/// A check is handed out whole, once its report is done, and in the order the checks were selected rather than the
/// order they finished in, so a verification says the same things in the same order however wide its pool is.
#[derive(Clone, Copy, Debug)]
pub enum GamedataEvent<'e> {
  /// The run is about to verify the project at `root`, running `checks` after the two that always run.
  Started {
    root: &'e Path,
    checks: &'e [GamedataVerificationType],
  },
  /// One check finished, with everything it found.
  CheckFinished {
    report: &'e GamedataVerificationCheckReport,
  },
}
