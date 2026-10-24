//! What the render commands that take more than an address were asked to do.

use serde::{Deserialize, Serialize};

/// A point of a viewport to name what its level draws under.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPickRequest {
  /// Css pixels from the viewport's left edge.
  pub x: f32,
  /// Css pixels from its top edge.
  pub y: f32,
  /// Whether glass is passed by, naming what stands behind it.
  pub is_through: bool,
}
