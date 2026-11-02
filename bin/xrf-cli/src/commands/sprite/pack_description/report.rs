use serde::Serialize;
use xrf_texture::{DescriptionSpriteOptions, DescriptionSpriteResult};

/// What a texture description run wrote, and where it read and wrote it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteDescriptionReport {
  base: String,
  description: String,
  destination: String,
  #[serde(flatten)]
  result: DescriptionSpriteResult,
}

impl SpriteDescriptionReport {
  pub fn new(options: &DescriptionSpriteOptions, result: &DescriptionSpriteResult) -> Self {
    Self {
      base: xrf_utils::to_portable_path_string(&options.base),
      description: xrf_utils::to_portable_path_string(&options.description),
      destination: xrf_utils::to_portable_path_string(&options.output_path),
      result: result.clone(),
    }
  }
}
