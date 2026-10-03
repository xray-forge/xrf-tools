use crate::lighting::render_sky::RenderSky;
use crate::pass::fullscreen_pipeline::texture_binding;
use crate::pass::layout_entries::texture_entry;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// What every pass drawing the weather's sky binds as its third group, as `shaders/common/sky.wgsl` declares it: both
/// keyframes' sky cubes, their irradiance cubes and their clouds, and the two samplers they are read through.
pub struct SkyBindings {
  layout: wgpu::BindGroupLayout,
  clamp: wgpu::Sampler,
  repeat: wgpu::Sampler,
}

impl SkyBindings {
  pub fn new(device: &wgpu::Device) -> Self {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let filtered: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: true };
    let cube: wgpu::TextureViewDimension = wgpu::TextureViewDimension::Cube;
    let flat: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2;
    let sampler = |binding: u32| wgpu::BindGroupLayoutEntry {
      binding,
      visibility: fragment,
      ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
      count: None,
    };
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("sky"),
      entries: &[
        texture_entry(0, fragment, filtered, cube),
        texture_entry(1, fragment, filtered, cube),
        texture_entry(2, fragment, filtered, cube),
        texture_entry(3, fragment, filtered, cube),
        texture_entry(4, fragment, filtered, flat),
        texture_entry(5, fragment, filtered, flat),
        sampler(6),
        sampler(7),
      ],
    });

    Self {
      layout,
      clamp: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("sky clamp"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      repeat: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("sky repeat"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
      }),
    }
  }

  pub fn get_layout(&self) -> &wgpu::BindGroupLayout {
    &self.layout
  }

  /// The sampler a sky cube is read through.
  pub fn get_clamp(&self) -> &wgpu::Sampler {
    &self.clamp
  }

  /// Binds a sky's textures as the cache holds them now, each slot its kind's placeholder until its file is up.
  pub fn create_bind_group(
    &self,
    device: &wgpu::Device,
    cache: &WeatherTextureCache,
    sky: &RenderSky,
  ) -> wgpu::BindGroup {
    let view = |reference: &Option<String>, kind: WeatherTextureKind| cache.get_view(reference.as_deref(), kind);

    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("sky"),
      layout: &self.layout,
      entries: &[
        texture_binding(0, view(&sky.textures[0], WeatherTextureKind::Cube)),
        texture_binding(1, view(&sky.textures[1], WeatherTextureKind::Cube)),
        texture_binding(2, view(&sky.environments[0], WeatherTextureKind::Cube)),
        texture_binding(3, view(&sky.environments[1], WeatherTextureKind::Cube)),
        texture_binding(4, view(&sky.clouds.textures[0], WeatherTextureKind::Flat)),
        texture_binding(5, view(&sky.clouds.textures[1], WeatherTextureKind::Flat)),
        wgpu::BindGroupEntry {
          binding: 6,
          resource: wgpu::BindingResource::Sampler(&self.clamp),
        },
        wgpu::BindGroupEntry {
          binding: 7,
          resource: wgpu::BindingResource::Sampler(&self.repeat),
        },
      ],
    })
  }
}
