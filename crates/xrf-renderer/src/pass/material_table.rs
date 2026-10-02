use wgpu::util::DeviceExt;

use crate::lighting::material_lut::{MATERIAL_LUT_DEPTH, MATERIAL_LUT_HEIGHT, MATERIAL_LUT_WIDTH, create_material_lut};

/// The engine's material table (`r2_material`), uploaded once and sampled by every pass that lights a surface.
pub struct MaterialTable {
  pub view: wgpu::TextureView,
  pub sampler: wgpu::Sampler,
}

impl MaterialTable {
  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let view: wgpu::TextureView = device
      .create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
          label: Some("material table"),
          size: wgpu::Extent3d {
            width: MATERIAL_LUT_WIDTH,
            height: MATERIAL_LUT_HEIGHT,
            depth_or_array_layers: MATERIAL_LUT_DEPTH,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D3,
          format: wgpu::TextureFormat::Rg8Unorm,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        },
        Default::default(),
        &create_material_lut(),
      )
      .create_view(&Default::default());
    let sampler: wgpu::Sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("material table"),
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      ..Default::default()
    });

    Self { view, sampler }
  }

  /// The table's texture and sampler as layout entries, at a binding and the one after it.
  pub fn get_layout_entries(binding: u32) -> [wgpu::BindGroupLayoutEntry; 2] {
    let visibility: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;

    [
      wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Texture {
          sample_type: wgpu::TextureSampleType::Float { filterable: true },
          view_dimension: wgpu::TextureViewDimension::D3,
          multisampled: false,
        },
        count: None,
      },
      wgpu::BindGroupLayoutEntry {
        binding: binding + 1,
        visibility,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
      },
    ]
  }

  /// The table's texture and sampler as bind group entries, at a binding and the one after it.
  pub fn get_entries(&self, binding: u32) -> [wgpu::BindGroupEntry<'_>; 2] {
    [
      wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::TextureView(&self.view),
      },
      wgpu::BindGroupEntry {
        binding: binding + 1,
        resource: wgpu::BindingResource::Sampler(&self.sampler),
      },
    ]
  }
}
