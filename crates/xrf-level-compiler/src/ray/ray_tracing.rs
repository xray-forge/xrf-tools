use std::path::Path;

use crate::ray::ray_backend::RayBackend;
use crate::ray::ray_gpu::RayGpu;

/// What traces a compiler's rays: the backend, and for the GPU backend with which DXC and on which GPU.
#[derive(Clone, Copy, Debug)]
pub struct RayTracing<'a> {
  pub backend: RayBackend,
  /// DXC's `dxcompiler.dll`, with which the GPU backend traces on D3D12; Vulkan alone without it.
  pub dxc: Option<&'a Path>,
  /// The GPU the GPU backend traces on; the high-performance one without it.
  pub gpu: Option<RayGpu>,
}
