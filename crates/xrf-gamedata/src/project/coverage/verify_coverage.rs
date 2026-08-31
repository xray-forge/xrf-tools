use std::time::{Duration, Instant};

use xrf_error::XrfResult;
use xrf_utils::format_path;
use xrf_vfs::XraySkippedMount;

use crate::project::coverage::verify_coverage_result::GamedataCoverageVerificationResult;
use crate::{Finding, GamedataFindingFactory, GamedataProject, GamedataProjectVerifyOptions, GamedataVerificationRule};

impl GamedataProject {
  /// Judges whether the run saw everything the project declared, which no content check can tell.
  ///
  /// Mounting tolerates a source it cannot open so one corrupt volume does not cost a tool the rest of an installation.
  /// That tolerance is only honest while it is visible: every count below is measured over what actually mounted, so a
  /// skipped source turns its assets into missing content or into nothing at all. This is where the record mounting kept
  /// becomes a verdict, beside the selection rather than inside it — folding it into a content check would let
  /// `--checks ltx` return success over an installation whose whole archive set went unread.
  ///
  /// Reports only sources a plan named and then failed to open. An `fsgame.ltx` alias omitted during planning is not the
  /// same fact: a real installation declares some thirty-five aliases, of which around ten are ever mounted, and the
  /// rest resolve inside the gamedata root already mounted or name writable state that was never an asset source.
  ///
  /// # Errors
  ///
  /// Infallible today, and fallible in signature because every check reports through the same seam.
  pub fn verify_coverage(
    &self,
    options: &GamedataProjectVerifyOptions,
  ) -> XrfResult<GamedataCoverageVerificationResult> {
    xrf_output::heading!(options.output, "Verify declared source coverage:");

    let started_at: Instant = Instant::now();
    let skipped: &[XraySkippedMount] = self.skipped_mounts();

    // Warned rather than merely stated, and before the checks below run, because it changes how every one of their
    // counts must be read.
    let findings: Vec<Finding> = skipped
      .iter()
      .map(|skipped| {
        xrf_output::warning!(
          options.output,
          "Declared source could not be opened: {} at {}: {}",
          skipped.origin,
          format_path(&skipped.path),
          skipped.reason
        );

        Self::skipped_mount_finding(skipped)
      })
      .collect();

    let duration: Duration = started_at.elapsed();

    xrf_output::info!(
      options.output,
      "Verified declared source coverage in {}, {} unopened source(s)",
      xrf_utils::format_duration(duration),
      skipped.len()
    );

    Ok(GamedataCoverageVerificationResult {
      duration,
      skipped_mounts_count: skipped.len(),
      findings,
    })
  }

  /// Names the source path as the subject and both how it was declared and why it failed in the message.
  ///
  /// All three, because a reader acting on this needs the alias to find the declaration, the path to find the source, and
  /// the reason to know whether to repair it or drop it.
  fn skipped_mount_finding(skipped: &XraySkippedMount) -> Finding {
    GamedataFindingFactory::for_asset(
      GamedataVerificationRule::CoverageSkippedMount,
      &skipped.path,
      format!(
        "Declared source '{}' could not be opened, so no result covers it: {}",
        skipped.origin, skipped.reason
      ),
    )
  }
}
