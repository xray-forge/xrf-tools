use std::path::{Path, PathBuf};

use serde::Serialize;
use xrf_app_core::TauriResult;
use xrf_vfs::XrayAsset;

use crate::describe::TextureFiles;
use crate::edit::TextureSaveTarget;

/// Where an edit of one texture would write, and what was there when the editor read it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureEditTargets {
  /// The `.thm` to write, whether or not one is there yet.
  pub descriptor: TextureSaveTarget,
  /// The `.dds` to replace when a re-encode is saved.
  pub texture: TextureSaveTarget,
}

impl TextureEditTargets {
  /// The two files an edit of `texture` writes, or nothing when the texture has no file on disk.
  ///
  /// # Errors
  ///
  /// Returns an error when either path is not Unicode, or exists and its metadata cannot be read.
  pub fn of(texture: &XrayAsset, descriptor: Option<&XrayAsset>) -> TauriResult<Option<Self>> {
    let Some(files) = Self::locate(texture, descriptor) else {
      return Ok(None);
    };

    Ok(Some(Self::of_paths(&files.descriptor, &files.texture)?))
  }

  /// The two files an edit writes, named by path.
  ///
  /// # Errors
  ///
  /// Returns an error when either path is not Unicode, or exists and its metadata cannot be read.
  pub fn of_paths(descriptor: &Path, texture: &Path) -> TauriResult<Self> {
    Ok(Self {
      descriptor: TextureSaveTarget::of_path(descriptor)?,
      texture: TextureSaveTarget::of_path(texture)?,
    })
  }

  /// The two files on disk an edit of `texture` writes, or nothing for a texture served out of an archive. The located
  /// descriptor when there is one, so a `.thm` a root override put somewhere else is the file that gets rewritten
  /// rather than a fresh one appearing beside the texture.
  pub fn locate(texture: &XrayAsset, descriptor: Option<&XrayAsset>) -> Option<TextureFiles> {
    let texture: PathBuf = texture.to_physical_path()?;

    Some(TextureFiles {
      descriptor: descriptor
        .and_then(XrayAsset::to_physical_path)
        .unwrap_or_else(|| TextureFiles::of(&texture).descriptor),
      texture,
    })
  }
}
