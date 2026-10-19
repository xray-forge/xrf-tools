/// What every water batch draws with: the nearest water's depth, the enhanced water's reflection and its resolve over
/// the history, then the surface as the engine draws it or enhanced, the enhanced one also writing ray reconstruction's
/// guides where it reads them, on a device whose targets hold them all (`GUIDED_BYTES`).
pub struct WaterPipelines {
  pub depth: wgpu::RenderPipeline,
  pub reflection: wgpu::RenderPipeline,
  pub reflection_resolve: wgpu::RenderPipeline,
  pub engine: wgpu::RenderPipeline,
  pub enhanced: wgpu::RenderPipeline,
  pub enhanced_guided: Option<wgpu::RenderPipeline>,
}

impl WaterPipelines {
  /// Bytes a sample of the guided surface's seven targets takes, past WebGPU's default 32: D3D12's and Vulkan's devices
  /// allow 64 and more.
  pub const GUIDED_BYTES: u32 = 36;
}
