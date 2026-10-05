use xrf_renderer_core::{GraphTexture, PassParameters, UniformBinding};

use crate::pass::upscale_uniform::UpscaleUniform;

/// What an upscale pass reads: the scene for EASU, the upscaled frame for RCAS, and the size and sharpness.
#[derive(PassParameters)]
#[parameters(group = 0)]
pub struct UpscaleParameters {
  #[texture(d2, unfilterable)]
  pub source: GraphTexture,
  #[uniform]
  pub upscale: UniformBinding<UpscaleUniform>,
}
