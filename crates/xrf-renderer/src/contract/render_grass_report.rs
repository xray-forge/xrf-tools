use serde::{Deserialize, Serialize};

/// What a view's grass came to on a recent frame: the tufts its cull kept and the most its lists hold.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderGrassReport {
  /// Tufts the cull kept, read back a frame or more late; past `capacity` the rest went undrawn.
  pub drawn: u32,
  /// Tufts the lists hold at most, none while no grass is drawn.
  pub capacity: u32,
}
