use std::path::PathBuf;

use xrf_job::JobOutcome;

/// What a generated bump pair came to.
#[derive(Clone, Debug, PartialEq)]
pub struct GenerateBumpResult {
  /// Whether the run reached the end of its work or stopped because it was asked to.
  ///
  /// A cancelled run wrote neither half. Both are encoded before either is written, so there is no point at which
  /// stopping could leave one half of a pair on disk for the other to be missing from.
  pub outcome: JobOutcome,
  /// The normals and gloss, written as `<name>_bump.dds`.
  pub bump: PathBuf,
  /// The compression error and the height, written as `<name>_bump#.dds`.
  pub companion: PathBuf,
  /// Mean gloss over the whole surface, in `0..=1`.
  pub gloss_power: f32,
}

impl GenerateBumpResult {
  /// Below this the SDK answers `-1000`, "Invalid gloss mask", and keeps the files it wrote.
  pub const MINIMUM_GLOSS_POWER: f32 = 0.1;

  /// Whether the gloss is so dark the surface will show no specular response worth having.
  ///
  /// A verdict rather than a failure, exactly as in the SDK: the pair is written either way, because a modder who
  /// meant to author a matte surface is not making a mistake, and one who did not wants to be told.
  pub fn is_gloss_too_dark(&self) -> bool {
    self.gloss_power < Self::MINIMUM_GLOSS_POWER
  }
}
