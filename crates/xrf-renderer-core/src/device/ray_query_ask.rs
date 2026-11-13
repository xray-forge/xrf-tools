use std::path::Path;

use crate::device::render_gpu::RenderGpu;

/// What a [`crate::RayQueryDevice`] is started with: DXC's `dxcompiler.dll`, which lets it trace on D3D12, and the GPU
/// to trace on; the high-performance one without it.
#[derive(Clone, Copy, Debug, Default)]
pub struct RayQueryAsk<'a> {
  pub dxc: Option<&'a Path>,
  pub gpu: Option<RenderGpu>,
}
