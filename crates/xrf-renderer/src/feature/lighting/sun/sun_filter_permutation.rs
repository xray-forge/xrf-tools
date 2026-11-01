use xrf_renderer_core::ShaderPermutation;

/// The texels the sun's cascade filter reaches each way, fixed in a pipeline so its loops compile to their count
/// (`common/sun_shadow`); below nought, the shadows' uniform says.
#[derive(Clone, Copy, Debug, ShaderPermutation)]
pub struct SunFilterPermutation {
  pub sun_filter_reach: i32,
}

impl SunFilterPermutation {
  /// The module whose pipelines it makes.
  pub const MODULE: &'static str = "frame/sun";
}
