use std::path::Path;

use serde::Serialize;
use xrf_texture::GenerateBumpResult;

/// What `dds make-bump` generated, and what it thinks of the gloss it was given.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DdsMakeBumpReport {
  /// The image the relief was read from.
  source: String,
  /// The normals and the gloss.
  bump: String,
  /// The compression error and the height.
  companion: String,
  /// Mean gloss over the whole surface, from 0 to 1.
  gloss_power: f32,
  /// Whether the gloss is too dark to show a specular response, which is a verdict rather than a failure.
  is_gloss_too_dark: bool,
}

impl DdsMakeBumpReport {
  pub fn new(source: &Path, result: &GenerateBumpResult) -> Self {
    Self {
      source: xrf_utils::to_portable_path_string(source),
      bump: xrf_utils::to_portable_path_string(&result.bump),
      companion: xrf_utils::to_portable_path_string(&result.companion),
      gloss_power: result.gloss_power,
      is_gloss_too_dark: result.is_gloss_too_dark(),
    }
  }
}
