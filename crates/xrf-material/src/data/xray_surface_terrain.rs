use serde::Serialize;

use crate::data::xray_surface_terrain_layer::XraySurfaceTerrainLayer;

/// What `B_BmmD` lays over a terrain's base under the deferred renderers (`Blender_BmmD_deferred.cpp`): four details
/// and their bumps, weighed by the four channels of a mask beside the base.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XraySurfaceTerrain {
  /// The mask, the base's reference with `_mask` after it.
  pub mask: String,
  /// The details its red, green, blue and alpha weigh, `R2-R` to `R2-A`.
  pub layers: [XraySurfaceTerrainLayer; 4],
}
