use crate::scene::texture::texture_cache::{CHECKER_SIZE, CHECKER_SQUARE, TextureCache};

impl TextureCache {
  /// Magenta and near black squares, as the WebGPU viewers stand in for a texture they cannot have.
  pub(super) fn create_checker(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
    use wgpu::util::DeviceExt;

    let texels: Vec<u8> = (0..CHECKER_SIZE * CHECKER_SIZE)
      .flat_map(|index| {
        let (x, y): (u32, u32) = (
          index % CHECKER_SIZE / CHECKER_SQUARE,
          index / CHECKER_SIZE / CHECKER_SQUARE,
        );

        if (x + y) % 2 == 0 {
          [255, 0, 255, 255]
        } else {
          [16, 16, 16, 255]
        }
      })
      .collect();

    device
      .create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
          label: Some("checker"),
          size: wgpu::Extent3d {
            width: CHECKER_SIZE,
            height: CHECKER_SIZE,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: wgpu::TextureFormat::Rgba8Unorm,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        },
        Default::default(),
        &texels,
      )
      .create_view(&Default::default())
  }

  pub(super) fn create_solid(device: &wgpu::Device, queue: &wgpu::Queue, color: [u8; 4]) -> wgpu::TextureView {
    use wgpu::util::DeviceExt;

    device
      .create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
          label: Some("solid"),
          size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: wgpu::TextureFormat::Rgba8Unorm,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        },
        Default::default(),
        &color,
      )
      .create_view(&Default::default())
  }

  /// Samples the textures with an anisotropy clamp, one for trilinear, made again where it changed.
  pub fn set_anisotropy(&mut self, device: &wgpu::Device, anisotropy: u16) {
    if anisotropy == self.anisotropy {
      return;
    }

    self.anisotropy = anisotropy;
    self.sampler = Self::create_sampler(device, anisotropy);
    self.bind_group = Self::create_bind_group(device, &self.layout, &self.sampler, &self.views);
  }

  /// Trilinear, anisotropic up to `anisotropy`, repeating, as the engine samples a level's surfaces.
  pub(super) fn create_sampler(device: &wgpu::Device, anisotropy: u16) -> wgpu::Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("textures"),
      address_mode_u: wgpu::AddressMode::Repeat,
      address_mode_v: wgpu::AddressMode::Repeat,
      address_mode_w: wgpu::AddressMode::Repeat,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      mipmap_filter: wgpu::MipmapFilterMode::Linear,
      anisotropy_clamp: anisotropy.max(1),
      ..Default::default()
    })
  }

  /// Binds every slot, the ones never filled to the missing texture, since an array is bound whole.
  pub(super) fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    views: &[wgpu::TextureView],
  ) -> wgpu::BindGroup {
    // Only the slots asked for: the array is partially bound, and every view bound is tracked by each pass binding it.
    let bound: Vec<&wgpu::TextureView> = views.iter().collect();

    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("textures"),
      layout,
      entries: &[
        wgpu::BindGroupEntry {
          binding: 0,
          resource: wgpu::BindingResource::TextureViewArray(&bound),
        },
        wgpu::BindGroupEntry {
          binding: 1,
          resource: wgpu::BindingResource::Sampler(sampler),
        },
      ],
    })
  }
}
