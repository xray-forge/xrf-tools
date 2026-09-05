use std::path::Path;

use serde::{Deserialize, Serialize};
use xrf_utils::format_path;
use xrf_vfs::{XrayAssetType, XrayLogicalPath, XrayMountPlan};

use crate::core::types::TauriResult;

/// Where a texture is named from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum TextureSource {
  /// A loose `.dds` or `.thm` on disk, named by its filesystem path.
  File { path: String },
  /// A texture of the roots, loose or archived, named by its engine reference such as `ston\ston_beton05`.
  Asset { reference: String },
}

impl TextureSource {
  pub fn label(&self) -> &str {
    match self {
      Self::File { path } => path,
      Self::Asset { reference } => reference,
    }
  }

  /// Returns the texture's filesystem path when its source provides one, for centring the roots on it.
  pub fn physical_path(&self) -> Option<&Path> {
    match self {
      Self::File { path } => Some(Path::new(path)),
      Self::Asset { .. } => None,
    }
  }

  /// The engine reference this source names.
  ///
  /// A file is named by its logical path inside the X-Ray root the VFS implies for it, which is the root the roots are
  /// centred on, so a reference derived here resolves in the same tree the describe then searches. A `.dds` and the
  /// `.thm` beside it share one reference, so either may be picked.
  ///
  /// # Errors
  ///
  /// Returns an error when no ancestor of the file is an X-Ray root, or when the file is neither a texture nor a
  /// descriptor under that root's textures directory.
  pub fn to_reference(&self) -> TauriResult<String> {
    match self {
      Self::Asset { reference } => Ok(reference.clone()),
      Self::File { path } => {
        let file: &Path = Path::new(path);
        let logical_path: XrayLogicalPath = XrayMountPlan::implied_logical_path(file).ok_or_else(|| {
          format!(
            "Texture '{}' is not inside an X-Ray root: no ancestor holds both a meshes and a textures directory",
            format_path(file)
          )
        })?;

        [XrayAssetType::Dds, XrayAssetType::Thm]
          .into_iter()
          .find_map(|kind| kind.get_rules()?.to_reference(&logical_path))
          .ok_or_else(|| {
            format!(
              "'{}' is neither a texture nor a texture descriptor under its root's textures directory",
              format_path(file)
            )
          })
      }
    }
  }
}
