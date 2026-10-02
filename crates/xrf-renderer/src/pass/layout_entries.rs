/// A storage buffer binding, read only or written.
pub fn storage_entry(binding: u32, visibility: wgpu::ShaderStages, is_written: bool) -> wgpu::BindGroupLayoutEntry {
  wgpu::BindGroupLayoutEntry {
    binding,
    visibility,
    ty: wgpu::BindingType::Buffer {
      ty: wgpu::BufferBindingType::Storage { read_only: !is_written },
      has_dynamic_offset: false,
      min_binding_size: None,
    },
    count: None,
  }
}

/// A uniform buffer binding.
pub fn uniform_entry(binding: u32, visibility: wgpu::ShaderStages) -> wgpu::BindGroupLayoutEntry {
  wgpu::BindGroupLayoutEntry {
    binding,
    visibility,
    ty: wgpu::BindingType::Buffer {
      ty: wgpu::BufferBindingType::Uniform,
      has_dynamic_offset: false,
      min_binding_size: None,
    },
    count: None,
  }
}

/// A texture binding read by `textureLoad` or sampled.
pub fn texture_entry(
  binding: u32,
  visibility: wgpu::ShaderStages,
  sample_type: wgpu::TextureSampleType,
  view_dimension: wgpu::TextureViewDimension,
) -> wgpu::BindGroupLayoutEntry {
  wgpu::BindGroupLayoutEntry {
    binding,
    visibility,
    ty: wgpu::BindingType::Texture {
      sample_type,
      view_dimension,
      multisampled: false,
    },
    count: None,
  }
}
