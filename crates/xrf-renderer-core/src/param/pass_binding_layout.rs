use crate::shader::ShaderType;

/// The layout entries `#[derive(PassParameters)]` builds, one kind of binding each. A binding is visible to every stage,
/// except a writable one, which a vertex shader may not hold.
pub struct PassBindingLayout;

impl PassBindingLayout {
  const EVERY_STAGE: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX_FRAGMENT.union(wgpu::ShaderStages::COMPUTE);
  const WRITABLE_STAGES: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT.union(wgpu::ShaderStages::COMPUTE);

  /// A uniform read at a dynamic offset into its upload ring.
  pub fn uniform<T: ShaderType>(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
      binding,
      visibility: Self::EVERY_STAGE,
      ty: wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: true,
        min_binding_size: wgpu::BufferSize::new(T::SIZE),
      },
      count: None,
    }
  }

  pub fn storage(binding: u32, is_writable: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
      binding,
      visibility: Self::get_visibility(is_writable),
      ty: wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Storage {
          read_only: !is_writable,
        },
        has_dynamic_offset: false,
        min_binding_size: None,
      },
      count: None,
    }
  }

  pub fn texture(
    binding: u32,
    sample_type: wgpu::TextureSampleType,
    view_dimension: wgpu::TextureViewDimension,
  ) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
      binding,
      visibility: Self::EVERY_STAGE,
      ty: wgpu::BindingType::Texture {
        sample_type,
        view_dimension,
        multisampled: false,
      },
      count: None,
    }
  }

  /// A binding array of `count` sampled textures alike.
  pub fn texture_array(
    binding: u32,
    sample_type: wgpu::TextureSampleType,
    view_dimension: wgpu::TextureViewDimension,
    count: u32,
  ) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
      count: std::num::NonZeroU32::new(count),
      ..Self::texture(binding, sample_type, view_dimension)
    }
  }

  pub fn storage_texture(
    binding: u32,
    access: wgpu::StorageTextureAccess,
    format: wgpu::TextureFormat,
    view_dimension: wgpu::TextureViewDimension,
    is_writable: bool,
  ) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
      binding,
      visibility: Self::get_visibility(is_writable),
      ty: wgpu::BindingType::StorageTexture {
        access,
        format,
        view_dimension,
      },
      count: None,
    }
  }

  pub fn sampler(binding: u32, ty: wgpu::SamplerBindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
      binding,
      visibility: Self::EVERY_STAGE,
      ty: wgpu::BindingType::Sampler(ty),
      count: None,
    }
  }

  fn get_visibility(is_writable: bool) -> wgpu::ShaderStages {
    if is_writable {
      Self::WRITABLE_STAGES
    } else {
      Self::EVERY_STAGE
    }
  }
}
