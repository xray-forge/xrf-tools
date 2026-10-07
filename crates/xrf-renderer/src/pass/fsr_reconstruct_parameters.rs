use xrf_renderer_core::{GraphTexture, PassParameters, ShaderAtomicU32, StorageArrayMut, UniformBinding};

use crate::pass::fsr_uniform::FsrUniform;

/// FSR 2's depth reconstruction: the depth and motion drawn, and the previous depth it scatters into.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 0)]
pub struct FsrReconstructParameters {
  #[uniform]
  pub fsr: UniformBinding<FsrUniform>,
  #[texture(d2, depth)]
  pub depth_target: GraphTexture,
  #[texture(d2, unfilterable)]
  pub motion_target: GraphTexture,
  #[storage]
  pub reconstructed: StorageArrayMut<ShaderAtomicU32>,
}
