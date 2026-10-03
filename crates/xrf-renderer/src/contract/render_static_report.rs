use serde::{Deserialize, Serialize};

use crate::contract::render_pool_use::RenderPoolUse;

/// How full a level's static draws' pools are, and what the camera's cull kept and occlusion hid.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderStaticReport {
  /// Slots, a draw each.
  pub slots: RenderPoolUse,
  /// Places static draws stand in.
  pub places: RenderPoolUse,
  /// Rows the instance cull tests, a spawned model's place each.
  pub rows: RenderPoolUse,
  /// Impostors of clumps of trees.
  pub lods: RenderPoolUse,
  /// Clusters static draws are made of.
  pub clusters: RenderPoolUse,
  /// Entries the camera's visible list held, of the room it has.
  pub surface_list: RenderPoolUse,
  /// Indirect draws the camera's static batches issue a frame.
  pub commands: u32,
  /// Clusters the camera kept, and their triangles.
  pub kept_clusters: u32,
  pub kept_triangles: u32,
  /// Clusters the frustum kept and the depth hid, and their triangles.
  pub occluded_clusters: u32,
  pub occluded_triangles: u32,
}
