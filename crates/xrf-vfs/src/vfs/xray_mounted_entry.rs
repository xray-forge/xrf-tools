use serde::Serialize;

use crate::XrayAsset;

/// One engine path as the mounted order answers it: the copy a lookup reaches, and the copies it hides.
///
/// The third of three listing shapes, and the only one that says what shadowing did.
/// [`crate::XrayVfs::list_entries`] answers winners and drops the rest, so nothing downstream can tell a path held
/// once from a path held four times; [`crate::XrayVfs::list_entries_all`] answers every copy flat, so a consumer that
/// wants the winners has to fold it again and guess the priority rule while doing so. A browser showing a mounted
/// installation needs both halves at once — which file the engine opens, and which files that decision hides — and
/// folding them here is what keeps that rule in the crate that owns mount priority.
///
/// Shadowing is not a [`crate::XrayPathCollision`]. A collision is two files inside *one* source claiming one identity,
/// where there is no priority to appeal to and one of them simply cannot be reached; shadowing happens *between*
/// mounts, where a loose file legitimately overrides an archived one and both are exactly where they should be.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct XrayMountedEntry {
  /// The copy a lookup for this path reaches.
  pub asset: XrayAsset,
  /// Unpacked bytes of the winner, as its own source records or measures them.
  pub size: u64,
  /// Copies no lookup reaches, in mount priority order behind the winner.
  pub shadowed: Vec<XrayAsset>,
}

impl XrayMountedEntry {
  /// Starts an entry at the copy that wins, with nothing yet known to hide behind it.
  pub(crate) fn new(asset: XrayAsset, size: u64) -> Self {
    Self {
      asset,
      size,
      shadowed: Vec::new(),
    }
  }

  /// The engine identity this entry answers for.
  pub fn get_logical_path(&self) -> &str {
    self.asset.get_logical_path().as_str()
  }

  /// Whether another mount holds a copy of this path that no lookup reaches.
  pub fn is_shadowing(&self) -> bool {
    !self.shadowed.is_empty()
  }
}
