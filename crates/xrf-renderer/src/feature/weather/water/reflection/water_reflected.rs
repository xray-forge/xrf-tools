use xrf_renderer_core::GraphTexture;

/// The enhanced water's reflection as a frame leaves it: blurred, which its rough surfaces read, and the history it
/// wrote, which its smooth ones read.
pub struct WaterReflected {
  pub blurred: GraphTexture,
  pub history: GraphTexture,
}
