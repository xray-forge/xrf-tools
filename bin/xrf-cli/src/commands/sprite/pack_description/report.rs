use serde::Serialize;
use xrf_texture::{SpriteDescriptionOptions, SpriteDescriptionResult};

/// What a texture description run wrote, and where it read and wrote it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteDescriptionReport {
  base: String,
  description: String,
  destination: String,
  #[serde(flatten)]
  result: SpriteDescriptionResult,
}

impl SpriteDescriptionReport {
  pub fn new(options: &SpriteDescriptionOptions, result: &SpriteDescriptionResult) -> Self {
    Self {
      base: xrf_utils::to_portable_path_string(&options.base),
      description: xrf_utils::to_portable_path_string(&options.description),
      destination: xrf_utils::to_portable_path_string(&options.output_path),
      result: result.clone(),
    }
  }
}
