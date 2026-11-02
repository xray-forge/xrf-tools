use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

/// What decides how much of a level's static geometry draws at a distance: the engine's screen area thresholds, each in
/// pixels of a 90 degree lens before `r__geometry_lod` scales them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderLodSettings {
  /// Whether distant trees are drawn as their impostors.
  pub is_impostors: bool,
  /// `r__geometry_lod`: every screen area threshold scales with it.
  #[setting(min = 0.1, max = 2)]
  pub geometry_lod: f32,
  /// `r_ssaLOD_A` and `r_ssaLOD_B`: a clump's impostor draws below the first, its trees above the second.
  #[setting(min = 16, max = 96)]
  pub ssa_a: f32,
  #[setting(min = 16, max = 64)]
  pub ssa_b: f32,
  /// `r_ssaDISCARD`: an instanced place smaller on screen than this is not drawn.
  #[setting(min = 1, max = 10)]
  pub ssa_discard: f32,
  /// `r_ssaGLOD_start` and `r_ssaGLOD_end`: a progressive mesh is whole above the first, coarsest below the second.
  #[setting(min = 128, max = 512)]
  pub ssa_glod_start: f32,
  #[setting(min = 16, max = 96)]
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
