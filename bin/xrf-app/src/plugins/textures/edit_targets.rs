use std::path::{Path, PathBuf};

use serde::Serialize;
use xrf_vfs::{XrayAsset, XrayAssetType};

use crate::core::types::TauriResult;
use crate::plugins::textures::file_stamp::TextureFileStamp;
use crate::plugins::textures::request::TextureSaveTarget;

/// Where an edit of one texture would write, and what was there when the editor read it.
///
/// Resolved by the command that located the files rather than derived by the frontend, for the same reason the
/// descriptor is: a path assembled in TypeScript out of a reference and a separator is a guess about where the VFS
/// found something, and the two disagree the moment a root is nested or a name is cased differently.
///
/// Absent for a texture served out of an archive, which has no file to replace at all.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureEditTargets {
  /// The `.thm` to write, whether or not one is there yet.
  ///
  /// Always present, because the editor can author a descriptor for a texture that has none: its
  /// [`TextureSaveTarget::expected`] is what says which of the two cases this is.
  pub descriptor: TextureSaveTarget,
  /// The `.dds` to replace when a re-encode is saved.
  pub texture: TextureSaveTarget,
}

impl TextureEditTargets {
  /// The two files an edit of `texture` writes, or nothing when the texture has no file on disk.
  ///
  /// The descriptor's path is the texture's with its extension swapped, which is the same rule
  /// [`XrayAssetType::Thm`] resolves a reference by: the engine looks for `<name>.thm` beside `<name>.dds` and nowhere
  /// else. Taking it from the located `.thm` instead would work only for the textures that already have one, which is
  /// exactly the case the editor does not need help with.
  ///
  /// # Errors
  ///
  /// Returns an error when either file exists and its metadata cannot be read.
  pub fn of(texture: &XrayAsset, descriptor: Option<&XrayAsset>) -> TauriResult<Option<Self>> {
    let Some(texture_path) = texture.to_physical_path() else {
      return Ok(None);
    };

    // The located descriptor when there is one, so a `.thm` a root override put somewhere else is the file that gets
    // rewritten rather than a fresh one appearing beside the texture.
    let descriptor_path: PathBuf = descriptor
      .and_then(XrayAsset::to_physical_path)
      .unwrap_or_else(|| to_descriptor_path(&texture_path));

    Ok(Some(Self {
      descriptor: to_target(&descriptor_path)?,
      texture: to_target(&texture_path)?,
    }))
  }
}

/// The descriptor that belongs beside a texture file.
fn to_descriptor_path(texture: &Path) -> PathBuf {
  texture.with_extension(
    XrayAssetType::Thm
      .get_rules()
      .map_or("thm", |rules| rules.extension.trim_start_matches('.')),
  )
}

fn to_target(path: &Path) -> TauriResult<TextureSaveTarget> {
  Ok(TextureSaveTarget {
    path: path.display().to_string(),
    expected: TextureFileStamp::read(path)?,
  })
}
