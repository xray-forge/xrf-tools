use serde::Serialize;
use xrf_texture::{DescriptionSpriteOptions, DescriptionSpriteResult};

/// What a texture description run wrote, and where it read and wrote it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SpriteDescriptionReport {
  base: String,
  description: String,
  destination: String,
  #[serde(flatten)]
  result: DescriptionSpriteResult,
}

impl SpriteDescriptionReport {
  pub(crate) fn new(options: &DescriptionSpriteOptions, result: &DescriptionSpriteResult) -> Self {
    Self {
      base: xrf_utils::format_portable_path(&options.base),
      description: xrf_utils::format_portable_path(&options.description),
      destination: xrf_utils::format_portable_path(&options.output_path),
      result: result.clone(),
    }
  }
}
