use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use xrf_utils::format_path;
use xrf_vfs::{XrayAssetType, XrayLogicalPath, XrayMountPlan};

use crate::plugins::textures::files::TextureFiles;

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

  /// Returns the named file's path when its source provides one, for centring the roots on it.
  pub fn physical_path(&self) -> Option<&Path> {
    match self {
      Self::File { path } => Some(Path::new(path)),
      Self::Asset { .. } => None,
    }
  }

  /// Returns the path of the `.dds` this source names, for a file that may be either half of the pair.
  pub fn to_texture_path(&self) -> Option<PathBuf> {
    Some(TextureFiles::of(self.physical_path()?).texture)
  }

  /// The engine reference this source names, or `None` for a file that names none.
  pub fn to_reference(&self) -> Option<String> {
    match self {
      Self::Asset { reference } => Some(reference.clone()),
      Self::File { path } => {
        let logical_path: XrayLogicalPath = XrayMountPlan::implied_logical_path(Path::new(path))?;

        [XrayAssetType::Dds, XrayAssetType::Thm]
          .into_iter()
          .find_map(|kind| kind.get_rules()?.to_reference(&logical_path))
      }
    }
  }

  /// What to call this texture on screen: its engine reference, or the file's own stem.
  pub fn to_label(&self) -> String {
    self.to_reference().unwrap_or_else(|| match self {
      Self::Asset { reference } => reference.clone(),
      Self::File { path } => Path::new(path).file_stem().map_or_else(
        || format_path(Path::new(path)).to_string(),
        |stem| stem.to_string_lossy().into_owned(),
      ),
    })
  }
}
