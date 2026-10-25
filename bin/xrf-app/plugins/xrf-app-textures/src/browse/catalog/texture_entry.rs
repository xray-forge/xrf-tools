use serde::Serialize;
use xrf_vfs::XrayAsset;

use crate::browse::TextureRole;
use crate::describe::TextureSource;

/// One texture name and the files the roots hold for it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureEntry {
  /// What to call this row: an engine reference such as `ston\ston_beton05`, or a loose file's path below its root.
  pub reference: String,
  /// How to open this row, which is the address the describe and every write take.
  pub source: TextureSource,
  pub role: TextureRole,
  /// The `.dds` the roots answer for this reference, winner first.
  pub texture: Option<XrayAsset>,
  /// The `.thm` the roots answer for this reference, winner first.
  pub descriptor: Option<XrayAsset>,
}

impl TextureEntry {
  /// A row of the game tree, opened by the reference it is listed under.
  pub(crate) fn of_reference(reference: String) -> Self {
    Self {
      role: TextureRole::of_reference(&reference),
      source: TextureSource::Asset {
        reference: reference.clone(),
      },
      reference,
      texture: None,
      descriptor: None,
    }
  }

  /// A row of a loose directory, opened by the file itself at `path`, which crosses to the page as it is.
  pub(crate) fn of_file(label: String, path: String) -> Self {
    Self {
      role: TextureRole::of_reference(&label),
      source: TextureSource::File { path },
      reference: label,
      texture: None,
      descriptor: None,
    }
  }

  /// What a loose row is keyed and labelled by: its path below the root, without the extension.
  pub(crate) fn loose_key(asset: &XrayAsset) -> Option<String> {
    let path: &str = asset.get_logical_path().as_str();

    if !asset.get_asset_type()?.get_rules()?.extension.matches(path) {
      return None;
    }

    path.rsplit_once('.').map(|(stem, _)| stem.to_owned())
  }
}
