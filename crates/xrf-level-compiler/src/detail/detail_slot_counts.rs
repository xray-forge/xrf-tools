use serde::{Deserialize, Serialize};

/// How many slots of a level's details the detail lighter met each way.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailSlotCounts {
  /// Planting nothing.
  pub unplanted: u32,
  /// Planting something, but at no density, and so cleared.
  pub emptied: u32,
  /// Planted, but touching no face of the ray model, and so left unlit.
  pub untouched: u32,
  pub lit: u32,
}
