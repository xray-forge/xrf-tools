use serde::Serialize;
use xrf_job::JobOutcome;
use xrf_texture::GenerateBumpResult;
use xrf_utils::to_portable_path_string;

/// What a generated pair came to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureMakeBumpOutcome {
  /// Whether the pair was written or the run stopped because it was asked to.
  pub outcome: JobOutcome,
  /// The normals and gloss, written as `<bump>.dds`.
  pub bump: String,
  /// The compression error and the height, written as `<bump>#.dds`.
  pub companion: String,
  /// Mean gloss over the whole surface, in `0..=1`.
  pub gloss_power: f32,
  /// Whether the gloss is too dark for the surface to show a specular response worth having.
  pub is_gloss_too_dark: bool,
}

impl TextureMakeBumpOutcome {
  /// What a generation came to, its files named as the page addresses them.
  pub fn of(result: &GenerateBumpResult) -> Self {
    Self {
      outcome: result.outcome,
      bump: to_portable_path_string(&result.bump),
      companion: to_portable_path_string(&result.companion),
      gloss_power: result.gloss_power,
      is_gloss_too_dark: result.is_gloss_too_dark(),
    }
  }
}
