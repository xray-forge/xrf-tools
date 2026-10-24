use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_app_core::jobs::JobResource;
use xrf_texture::TextureEncodingQuality;

use crate::edit::TextureDescriptorForm;

/// What a texture rebuild was asked to do.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TexturesBuildRequest {
  /// Path of the `.dds` to write, which is the file beside the descriptor.
  pub destination: String,
  /// Path of the image to encode, of whatever kind `image` decodes.
  pub source: String,
  /// The descriptor to read as a recipe, as the editor currently has it rather than as it is on disk.
  pub descriptor: TextureDescriptorForm,
  pub quality: TextureEncodingQuality,
}

impl TexturesBuildRequest {
  /// The file this build would write, as a write claim.
  pub fn to_resources(&self) -> Vec<JobResource> {
    vec![JobResource::file(&self.destination)]
  }

  /// The `.dds` this build writes.
  pub fn to_destination(&self) -> PathBuf {
    PathBuf::from(&self.destination)
  }
}
