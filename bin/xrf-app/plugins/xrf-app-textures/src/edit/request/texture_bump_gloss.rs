use serde::{Deserialize, Serialize};
use xrf_app_core::TauriResult;
use xrf_texture::{GenerateBumpGloss, ImageFileReader};
use xrf_utils::error_to_string;

/// Where a generated bump pair takes its gloss from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum TextureBumpGloss {
  /// The height map's own alpha, where the SDK's bump source carries its gloss.
  HeightAlpha,
  /// One level for the whole surface, from 0 to 1, for a surface authored without a gloss.
  Constant { level: f32 },
  /// Path of a gloss mask, averaged across its colour channels.
  Mask { path: String },
}

impl TextureBumpGloss {
  /// The gloss the generator takes, a mask read from disk.
  ///
  /// # Errors
  ///
  /// Returns an error for a constant outside the 0 to 1 range a gloss is measured in, or a mask that cannot be read.
  pub fn to_gloss(&self) -> TauriResult<GenerateBumpGloss> {
    match self {
      Self::HeightAlpha => Ok(GenerateBumpGloss::HeightAlpha),
      Self::Constant { level } if (0.0..=1.0).contains(level) => Ok(GenerateBumpGloss::Constant(*level)),
      Self::Constant { level } => Err(format!(
        "Gloss level {level} is outside the 0 to 1 range a gloss is measured in"
      )),
      Self::Mask { path } => Ok(GenerateBumpGloss::Mask(
        ImageFileReader::read_rgba(path).map_err(error_to_string)?,
      )),
    }
  }
}
