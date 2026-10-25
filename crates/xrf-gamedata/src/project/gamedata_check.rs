use xrf_error::XrfResult;

use crate::{GamedataCheckResult, GamedataProject, GamedataProjectVerifyOptions};

/// One check a verification runs: what it judges is its own, and the project only lends it the assets to judge.
///
/// Every check takes the same two inputs and answers one result, so `GamedataVerificationType::run` builds and runs any
/// of them the same way, and a check's behaviour lives on its own type rather than on the project it reads.
pub(crate) trait GamedataCheck<'a> {
  type Result: GamedataCheckResult;

  fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self;

  /// Runs the check over the project.
  ///
  /// # Errors
  ///
  /// Returns a cancellation error when the job is stopped, and otherwise an error only where the check could not be
  /// performed at all; what it finds wrong with the assets is in the result.
  fn verify(&self) -> XrfResult<Self::Result>;
}
