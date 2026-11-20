use xrf_math::Matrix4x4;

/// Where a model's impostor was stood: its first placement's `MT_LOD`, by its visual's index, and that placement.
pub(crate) struct LevelTreeImpostorFrame {
  pub(crate) reference: String,
  pub(crate) lod: u32,
  pub(crate) placement: Matrix4x4,
}
