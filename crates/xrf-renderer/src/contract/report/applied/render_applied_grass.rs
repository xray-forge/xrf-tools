use serde::{Deserialize, Serialize};

/// The grass as planted.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAppliedGrass {
  /// Metres around the camera it is planted to, in whole slots.
  pub radius: f32,
  /// How far apart a slot's candidates stand, held to the engine's bounds.
  pub density: f32,
  pub height: f32,
  /// Tufts the lists hold at most.
  pub tufts: u32,
  /// Tufts the radius and density would plant: more than `tufts` where a GPU buffer cannot hold them all.
  pub wanted: u32,
}
