use serde::{Deserialize, Serialize};

/// What one viewport draws its scene with, as its viewer's toolbar sets it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderViewOptions {
  /// Whether surfaces wear their textures, else their flat colours.
  pub is_textured: bool,
  /// Whether bump textures bend the normal.
  pub is_bumped: bool,
  /// How far the baked hemisphere darkens the ambient: zero for not at all.
  pub hemi_strength: f32,
  /// Whether what the last frame's depth hides is left undrawn.
  pub is_occlusion_culled: bool,
  /// Whether distant trees are drawn as their impostors.
  pub is_impostors: bool,
  /// `r__geometry_lod`: every screen area threshold scales with it.
  pub geometry_lod: f32,
}

impl Default for RenderViewOptions {
  fn default() -> Self {
    Self {
      is_textured: true,
      is_bumped: true,
      hemi_strength: 1.0,
      is_occlusion_culled: true,
      is_impostors: true,
      geometry_lod: 0.75,
    }
  }
}
