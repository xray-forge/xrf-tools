use xrf_renderer_core::ShaderPermutation;

/// The rays traced occlusion casts a pixel.
#[derive(Clone, Copy, Debug, ShaderPermutation)]
pub struct RtaoPermutation {
  pub rays: u32,
}

impl RtaoPermutation {
  /// The module whose pipelines it makes.
  pub const MODULE: &'static str = "frame/rtao";
}
