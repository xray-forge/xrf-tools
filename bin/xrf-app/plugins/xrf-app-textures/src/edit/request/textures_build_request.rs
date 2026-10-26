use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_app_core::TauriResult;
use xrf_app_core::jobs::JobResource;
use xrf_chunk::XRayByteOrder;
use xrf_texture::{BuildTextureNormalMap, BuildTextureRecipe, ImageFileReader, TextureEncodingQuality};
use xrf_thm::ThmFile;
use xrf_utils::{error_to_string, format_path};

use crate::describe::TextureFiles;
use crate::edit::TextureDescriptorForm;

/// What a texture rebuild was asked to do.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TexturesBuildRequest {
  /// Path of the `.dds` to write, which is the file beside the descriptor.
  pub destination: String,
  /// Path of the image to encode, of whatever kind `image` decodes; for a bump descriptor, the height in its colour and
  /// the gloss in its alpha.
  pub source: String,
  /// The descriptor to read as a recipe, as the editor currently has it rather than as it is on disk.
  pub descriptor: TextureDescriptorForm,
  /// Path of the texture a bump descriptor names as its external normal map, whose own `.thm` beside it says what it
  /// is; absent for any other.
  pub normal_map: Option<String>,
  pub quality: TextureEncodingQuality,
}

impl TexturesBuildRequest {
  /// The files this build would write, as write claims: a bump descriptor's companion beside the texture. A descriptor
  /// the build refuses claims the texture alone, since it writes nothing.
  pub fn to_resources(&self) -> Vec<JobResource> {
    let destination: PathBuf = self.to_destination();

    BuildTextureRecipe::of(&self.descriptor.to_descriptor(Some(ThmFile::new_texture())))
      .map_or_else(
        |_| vec![destination.clone()],
        |recipe| recipe.layout.get_written_paths(&destination),
      )
      .iter()
      .map(JobResource::file)
      .collect()
  }

  /// The `.dds` this build writes.
  pub fn to_destination(&self) -> PathBuf {
    PathBuf::from(&self.destination)
  }

  /// The external normal map the request names, its pixels and its descriptor read.
  ///
  /// # Errors
  ///
  /// Returns an error when the map or its descriptor cannot be read.
  pub fn to_normal_map(&self) -> TauriResult<Option<BuildTextureNormalMap>> {
    let Some(path) = &self.normal_map else {
      return Ok(None);
    };
    let files: TextureFiles = TextureFiles::of(Path::new(path));

    Ok(Some(BuildTextureNormalMap {
      descriptor: ThmFile::read_from_path::<XRayByteOrder, _>(&files.descriptor).map_err(|error| {
        format!(
          "Cannot read the descriptor of normal map {}: {error}",
          format_path(&files.descriptor)
        )
      })?,
      image: ImageFileReader::read_rgba(&files.texture).map_err(error_to_string)?,
    }))
  }
}
