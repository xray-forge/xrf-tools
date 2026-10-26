use serde::Serialize;

use crate::XrayAsset;

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
