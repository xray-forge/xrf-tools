use std::path::{Path, PathBuf};

use xrf_vfs::XrayAssetType;

/// The two files one texture is: the `.dds` its pixels are in and the `.thm` that describes them, each named from the
/// other by swapping the extension.
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
      texture: Self::sibling(path, XrayAssetType::Dds),
      descriptor: Self::sibling(path, XrayAssetType::Thm),
    }
  }

  /// The path of the sibling of this kind: the same stem, carrying this kind's extension.
  fn sibling(path: &Path, asset_type: XrayAssetType) -> PathBuf {
    asset_type.get_rules().map_or_else(
      || path.to_path_buf(),
      |rules| path.with_extension(rules.extension.as_str()),
    )
  }
}
