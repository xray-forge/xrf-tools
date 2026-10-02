/// What the renderer refuses to start without: compressed textures as stored, GPU-driven indirect drawing, bindless
/// texture arrays and dual-source blending. Every D3D12 feature level 12_0 GPU from about 2016 has them.
pub const REQUIRED_FEATURES: wgpu::Features = wgpu::Features::TEXTURE_COMPRESSION_BC
  .union(wgpu::Features::INDIRECT_FIRST_INSTANCE)
  .union(wgpu::Features::MULTI_DRAW_INDIRECT_COUNT)
  .union(wgpu::Features::TEXTURE_BINDING_ARRAY)
  .union(wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY)
  .union(wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING)
  .union(wgpu::Features::IMMEDIATES)
  .union(wgpu::Features::FLOAT32_FILTERABLE)
  .union(wgpu::Features::DUAL_SOURCE_BLENDING);

/// What the renderer uses where present: GPU timings of each pass.
pub const OPTIONAL_FEATURES: wgpu::Features = wgpu::Features::TIMESTAMP_QUERY
  .union(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS)
  .union(wgpu::Features::TIMESTAMP_QUERY_INSIDE_PASSES)
  .union(wgpu::Features::PIPELINE_STATISTICS_QUERY);

/// The required features an adapter lacks, empty when it has every one.
pub fn get_missing_features(features: wgpu::Features) -> wgpu::Features {
  REQUIRED_FEATURES.difference(features)
}
