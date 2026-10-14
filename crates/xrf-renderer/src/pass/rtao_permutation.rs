use xrf_renderer_core::ShaderPermutation;

/// The rays traced occlusion casts a pixel (`frame/rtao`).
#[derive(Clone, Copy, Debug, ShaderPermutation)]
pub struct RtaoPermutation {
  pub rays: u32,
}
