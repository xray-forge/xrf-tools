use serde::{Deserialize, Serialize};

use crate::contract::report::scene::render_drawable_kind::RenderDrawableKind;

/// What a drawable kind beside the static scene came to in a view on a recent frame: what its cull kept and the most
/// its lists hold.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderDrawableReport {
  pub kind: RenderDrawableKind,
  /// What the cull kept, read back a frame or more late; past `capacity` the rest went undrawn.
  pub drawn: u32,
  /// What the lists hold at most, none while the kind draws nothing.
  pub capacity: u32,
}
