use serde::{Deserialize, Serialize};

/// What of a level is drawn under a point of a viewport, and where the ray from the eye met it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLevelHit {
  pub sector: u32,
  /// The shader table entry drawing it.
  pub shader_id: u32,
  /// The sector's instanced mesh it is one place of, or none for its baked geometry.
  pub mesh: Option<u32>,
  /// Which place of the mesh it is, or none for the baked geometry.
  pub place: Option<u32>,
  /// Whether it is a clump of trees drawn as its impostor.
  pub is_impostor: bool,
  /// Where the ray met it, in renderer space.
  pub point: [f32; 3],
}
