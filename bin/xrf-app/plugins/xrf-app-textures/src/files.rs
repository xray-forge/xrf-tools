//! The two files one texture is, and the rule that names one from the other.

use std::path::{Path, PathBuf};

use xrf_vfs::XrayAssetType;

/// The `.dds` a texture's pixels are in and the `.thm` that describes them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureFiles {
  /// Where the pixels are.
  pub texture: PathBuf,
  /// Where the descriptor is, whether or not one has been written yet.
  pub descriptor: PathBuf,
}

impl TextureFiles {
  /// The two files that belong to whichever half `path` names.
  pub fn of(path: &Path) -> Self {
    Self {
      texture: to_sibling(path, XrayAssetType::Dds),
      descriptor: to_sibling(path, XrayAssetType::Thm),
    }
  }
}

/// The path of the sibling of this kind: the same stem, carrying this kind's extension.
fn to_sibling(path: &Path, asset_type: XrayAssetType) -> PathBuf {
  asset_type.get_rules().map_or_else(
    || path.to_path_buf(),
    |rules| path.with_extension(rules.extension.as_str()),
  )
}
