use xrf_renderer_core::ShaderPermutation;

/// The bitmask search at one quality (`frame/vbao`): its slices, the steps along each, and the steps a lit search takes
/// past the occlusion's radius.
#[derive(Clone, Copy, Debug, ShaderPermutation)]
pub struct VbaoPermutation {
  pub slices: u32,
  pub steps: u32,
  pub far_steps: u32,
}
