use serde::{Deserialize, Serialize};

/// How many slots of a level's details the detail lighter met each way.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailSlotCounts {
  /// Planting nothing.
  pub unplanted: usize,
  /// Planting something, but at no density, and so cleared.
  pub emptied: usize,
  /// Planted, but touching no face of the ray model, and so left unlit.
  pub untouched: usize,
  pub lit: usize,
}
