use std::time::{Duration, Instant};

use xrf_error::XrfResult;
use xrf_vfs::XrayPathCollision;

use crate::project::collisions::verify_collisions_result::GamedataCollisionsVerificationResult;
use crate::project::gamedata_check::GamedataCheck;
use crate::{Finding, GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// Judges the project's own inputs, which no asset check can see.
///
/// Two files of one source normalizing to a single engine identity leave one of them unreachable: the game never loads
/// it, whatever the author intended. The mounted world records that while indexing, and this is where the record becomes
/// a verdict — folding it into a content check would let `--checks ltx` hide an unreachable texture.
pub(crate) struct CollisionsVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  project: &'a GamedataProject,
}

impl<'a> GamedataCheck<'a> for CollisionsVerifier<'a> {
  type Result = GamedataCollisionsVerificationResult;

  fn new(project: &'a GamedataProject, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self { options, project }
  }

  fn verify(&self) -> XrfResult<GamedataCollisionsVerificationResult> {
    self.options.job.check_cancelled()?;

    xrf_output::heading!(self.options.output, "Verify path collisions:");

    let started_at: Instant = Instant::now();
    let collisions: Vec<XrayPathCollision> = self.project.collisions();

    let findings: Vec<Finding> = collisions
      .iter()
      .take_while(|_| !self.options.job.is_cancelled())
      .map(|collision| {
        xrf_output::info!(
          self.options.output,
          "File cannot be reached: {}, {} answers '{}'",
          collision.unreachable,
          collision.kept,
          collision.logical_path
        );

        Self::path_collision_finding(collision)
      })
      .collect();

    self.options.job.check_cancelled()?;

    let duration: Duration = started_at.elapsed();

    xrf_output::info!(
      self.options.output,
      "Verified gamedata path collisions in {}, {} unreachable file(s)",
      xrf_utils::format_duration(duration),
      collisions.len()
    );

    Ok(GamedataCollisionsVerificationResult {
      duration,
      unreachable_files_count: collisions.len(),
      findings,
    })
  }
}

impl CollisionsVerifier<'_> {
  /// Names the claimed identity as the subject and both sites in the message.
  ///
  /// Both, because either one alone leaves the reader unable to act: removing or renaming the right file means knowing
  /// which of the two spellings the source already answers with. Sites are rendered portably, so the same project
  /// reports the same finding on every platform.
  fn path_collision_finding(collision: &XrayPathCollision) -> Finding {
    GamedataFindingFactory::for_asset(
      GamedataVerificationRule::CollisionsUnreachable,
      collision.logical_path.as_str(),
      format!(
        "File '{}' cannot be reached, '{}' claims this path",
        collision.unreachable.to_portable_string(),
        collision.kept.to_portable_string()
      ),
    )
  }
}
