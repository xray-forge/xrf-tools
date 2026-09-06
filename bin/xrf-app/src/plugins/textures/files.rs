//! The two files one texture is, and the rule that names one from the other.

use std::path::{Path, PathBuf};

use xrf_vfs::XrayAssetType;

/// The `.dds` a texture's pixels are in and the `.thm` that describes them.
///
/// One rule with one owner, because it is the rule the whole plugin rests on and every surface needs a different half
/// of it: a description reads both, a save writes both, a comparison decodes the texture, and a person opens whichever
/// of the two they happened to pick. Spelled per caller it drifts - and it did, in four places with three different
/// answers for a kind whose extension could not be named, one of which handed a chunked descriptor to a dds reader.
///
/// The rule itself is the engine's: `<name>.thm` sits beside `<name>.dds` and nowhere else, which is also how
/// [`XrayAssetType::Thm`] resolves a reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureFiles {
  /// Where the pixels are.
  pub texture: PathBuf,
  /// Where the descriptor is, whether or not one has been written yet.
  pub descriptor: PathBuf,
}

impl TextureFiles {
  /// The two files that belong to whichever half `path` names.
  ///
  /// Takes either half on purpose. A person opens a texture by picking a file, and which of the two they pick is not
  /// a decision they should have to get right - so no caller has to ask which one it was given.
  pub fn of(path: &Path) -> Self {
    Self {
      texture: to_sibling(path, XrayAssetType::Dds),
      descriptor: to_sibling(path, XrayAssetType::Thm),
    }
  }
}

/// The path of the sibling of this kind: the same stem, carrying this kind's extension.
///
/// `Path::with_extension` refuses the leading dot the rules carry it with, hence the trim. A kind that names no
/// extension answers the path unchanged rather than inventing one, so the worst case is a caller reading the file it
/// was handed - never a path with the extension stripped off, which is what one of the four spellings produced.
fn to_sibling(path: &Path, asset_type: XrayAssetType) -> PathBuf {
  asset_type.get_rules().map_or_else(
    || path.to_path_buf(),
    |rules| path.with_extension(rules.extension.trim_start_matches('.')),
  )
}
