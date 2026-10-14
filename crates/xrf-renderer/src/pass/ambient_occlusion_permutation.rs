use xrf_renderer_core::ShaderPermutation;

/// The ambient occlusion's search at one quality: its slices and the steps along each.
#[derive(Clone, Copy, Debug, ShaderPermutation)]
pub struct AmbientOcclusionPermutation {
  pub slices: u32,
  pub steps: u32,
}

impl AmbientOcclusionPermutation {
  /// The module whose pipelines it makes.
  pub const MODULE: &'static str = "frame/ambient_occlusion";
}
