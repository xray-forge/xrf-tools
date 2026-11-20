use crate::feature::tracing::ray_guides::ray_guides_pass::RayGuidesPass;

/// What every water batch draws with: the nearest water's depth, the enhanced water's reflection and its resolve over
/// the history, then the surface as the engine draws it or enhanced, the enhanced one also writing ray reconstruction's
/// guides where it reads them, on a device whose targets hold them all (`GUIDED_BYTES`).
pub(crate) struct WaterPipelines {
  pub depth: wgpu::RenderPipeline,
  pub reflection: wgpu::RenderPipeline,
  pub reflection_resolve: wgpu::RenderPipeline,
  pub engine: wgpu::RenderPipeline,
  pub enhanced: wgpu::RenderPipeline,
  pub enhanced_guided: Option<wgpu::RenderPipeline>,
}

impl WaterPipelines {
  /// The guides the enhanced water writes past the surface's own targets, over the guides pass's: its specular albedo,
  /// its normal with its roughness, and its reflected ray's length.
  pub(crate) const GUIDES: [wgpu::TextureFormat; 3] = [
    RayGuidesPass::SPECULAR_ALBEDO,
    RayGuidesPass::NORMALS,
    RayGuidesPass::HIT_DISTANCE,
  ];
  /// Bytes a sample of the guided surface's targets takes by WebGPU's count, past its default 32: D3D12's and Vulkan's
  /// devices allow 64 and more.
  pub(crate) const GUIDED_BYTES: u32 = 38;
}
