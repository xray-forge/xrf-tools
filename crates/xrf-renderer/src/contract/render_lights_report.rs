use serde::{Deserialize, Serialize};

use crate::contract::render_pool_use::RenderPoolUse;

/// What a level's local lights came to on the last frame counted.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLightsReport {
  /// Lights standing in view and lit.
  pub in_view: u32,
  /// Of them, lights lit with their shadows.
  pub shadowed: u32,
  /// Lights in view past the most a frame holds, the farthest.
  pub excess: u32,
  /// Texels of the shadow atlas held, of its whole.
  pub atlas: RenderPoolUse,
  /// Clusters of the view more lights reached than one holds, and the lights they left out.
  pub full_clusters: u32,
  pub dropped: u32,
}
