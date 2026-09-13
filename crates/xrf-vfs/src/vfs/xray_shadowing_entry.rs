use serde::Serialize;

use crate::XrayAsset;
use crate::vfs::xray_mounted_entry::XrayMountedEntry;

/// One copy of an engine path that no lookup reaches, and how large it is.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct XrayShadowedCopy {
  /// The copy no lookup reaches.
  pub asset: XrayAsset,
  /// Unpacked bytes of this copy, as its own source records or measures them.
  pub size: u64,
}

impl XrayShadowedCopy {
  pub(crate) fn new(asset: XrayAsset, size: u64) -> Self {
    Self { asset, size }
  }
}

/// One engine path as the mounted order answers it: the copy a lookup reaches, and the copies it hides.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct XrayShadowingEntry {
  /// The copy a lookup for this path reaches, and its size.
  pub entry: XrayMountedEntry,
  /// Copies no lookup reaches, in mount priority order behind the winner.
  pub shadowed: Vec<XrayShadowedCopy>,
}

impl XrayShadowingEntry {
  /// Starts an entry at the copy that wins, with nothing yet known to hide behind it.
  pub(crate) fn new(asset: XrayAsset, size: u64) -> Self {
    Self {
      entry: XrayMountedEntry::new(asset, size),
      shadowed: Vec::new(),
    }
  }

  /// The engine identity this entry answers for.
  pub fn get_logical_path(&self) -> &str {
    self.entry.get_logical_path()
  }

  /// The copy a lookup for this path reaches.
  pub fn get_asset(&self) -> &XrayAsset {
    &self.entry.asset
  }

  /// Unpacked bytes of the winner.
  pub fn get_size(&self) -> u64 {
    self.entry.size
  }

  /// Whether another mount holds a copy of this path that no lookup reaches.
  pub fn is_shadowing(&self) -> bool {
    !self.shadowed.is_empty()
  }

  /// Unpacked bytes held by the copies no lookup reaches.
  ///
  /// What an override arrangement costs over answering every path once, which is the figure the winner's own size
  /// cannot express.
  pub fn get_shadowed_size(&self) -> u64 {
    self.shadowed.iter().map(|copy| copy.size).sum()
  }
}
