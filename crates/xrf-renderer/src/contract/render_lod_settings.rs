use serde::{Deserialize, Serialize};

/// What decides how much of a level's static geometry draws at a distance: the engine's screen area thresholds, each in
/// pixels of a 90 degree lens before `r__geometry_lod` scales them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLodSettings {
  /// Whether distant trees are drawn as their impostors.
  pub is_impostors: bool,
  /// `r__geometry_lod`: every screen area threshold scales with it.
  pub geometry_lod: f32,
  /// `r_ssaLOD_A` and `r_ssaLOD_B`: a clump's impostor draws below the first, its trees above the second.
  pub ssa_a: f32,
  pub ssa_b: f32,
  /// `r_ssaDISCARD`: an instanced place smaller on screen than this is not drawn.
  pub ssa_discard: f32,
  /// `r_ssaGLOD_start` and `r_ssaGLOD_end`: a progressive mesh is whole above the first, coarsest below the second.
  pub ssa_glod_start: f32,
  pub ssa_glod_end: f32,
}

impl Default for RenderLodSettings {
  /// The engine's own.
  fn default() -> Self {
    Self {
      is_impostors: true,
      geometry_lod: 0.75,
      ssa_a: 64.0,
      ssa_b: 48.0,
      ssa_discard: 3.5,
      ssa_glod_start: 256.0,
      ssa_glod_end: 64.0,
    }
  }
}
