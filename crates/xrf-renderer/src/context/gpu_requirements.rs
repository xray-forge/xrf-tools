use xrf_env_flags::EnvFlags;

/// What the renderer refuses to start without: compressed textures as stored, GPU-driven indirect drawing, bindless
/// texture arrays and dual-source blending. Every D3D12 feature level 12_0 GPU from about 2016 has them.
pub const REQUIRED_FEATURES: wgpu::Features = wgpu::Features::TEXTURE_COMPRESSION_BC
  .union(wgpu::Features::INDIRECT_FIRST_INSTANCE)
  .union(wgpu::Features::TEXTURE_BINDING_ARRAY)
  .union(wgpu::Features::PARTIALLY_BOUND_BINDING_ARRAY)
  .union(wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING)
  .union(wgpu::Features::IMMEDIATES)
  .union(wgpu::Features::FLOAT32_FILTERABLE)
  .union(wgpu::Features::DUAL_SOURCE_BLENDING);

/// What the renderer uses where present: GPU timings of each pass.
pub const OPTIONAL_FEATURES: wgpu::Features = wgpu::Features::TIMESTAMP_QUERY
  .union(wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS)
  .union(wgpu::Features::PIPELINE_CACHE);

/// What the renderer traces rays with where present: hardware ray queries against the level's acceleration structures,
/// still experimental in wgpu. `XRF_RENDER_RAY_QUERY` switched off leaves them off.
pub const RAY_QUERY_FEATURES: wgpu::Features = wgpu::Features::EXPERIMENTAL_RAY_QUERY;

/// The ray query features to ask an adapter for: all of them where it has them and they are not switched off.
pub fn get_ray_query_features(features: wgpu::Features) -> wgpu::Features {
  let is_off: bool = EnvFlags::RENDER_RAY_QUERY.read_switch() == Some(false);

  if is_off || !features.contains(RAY_QUERY_FEATURES) {
    wgpu::Features::empty()
  } else {
    RAY_QUERY_FEATURES
  }
}

/// The required features an adapter lacks, empty when it has every one.
pub fn get_missing_features(features: wgpu::Features) -> wgpu::Features {
  REQUIRED_FEATURES.difference(features)
}

/// Whether `device` traces rays against the scene's geometry.
pub fn can_trace(device: &wgpu::Device) -> bool {
  device.features().contains(RAY_QUERY_FEATURES)
}
