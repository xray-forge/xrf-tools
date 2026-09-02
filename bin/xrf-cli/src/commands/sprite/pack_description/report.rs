use serde::Serialize;
use xrf_texture::PackDescriptionOptions;

/// What a texture description run was pointed at.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteDescriptionReport {
  base: String,
  description: String,
  destination: String,
  files: Vec<String>,
  is_strict: bool,
}

impl SpriteDescriptionReport {
  pub fn new(options: &PackDescriptionOptions) -> Self {
    Self {
      base: xrf_utils::to_portable_path_string(&options.base),
      description: xrf_utils::to_portable_path_string(&options.description),
      destination: xrf_utils::to_portable_path_string(&options.output_path),
      files: options.files.clone(),
      is_strict: options.is_strict,
    }
  }
}
