use xrf_renderer_core::ShaderPermutation;

/// The ambient occlusion's search at one quality (`frame/ambient_occlusion`): its slices and the steps along each.
#[derive(Clone, Copy, Debug, ShaderPermutation)]
pub struct AmbientOcclusionPermutation {
  pub slices: u32,
  pub steps: u32,
}
