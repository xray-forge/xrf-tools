use serde::Serialize;

use crate::XrayAsset;

/// One engine path as the mounted order answers it: the copy a lookup reaches, and how large it is.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct XrayMountedEntry {
  /// The copy a lookup for this path reaches.
  pub asset: XrayAsset,
  /// Unpacked bytes of the winner, as its own source records or measures them.
  pub size: u64,
}

impl XrayMountedEntry {
  /// One entry at the copy that wins.
  pub(crate) fn new(asset: XrayAsset, size: u64) -> Self {
    Self { asset, size }
  }

  /// The engine identity this entry answers for.
  pub fn get_logical_path(&self) -> &str {
    self.asset.get_logical_path().as_str()
  }
}
