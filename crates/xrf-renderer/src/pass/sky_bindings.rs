use xrf_renderer_core::{FrameGraph, GraphBindings, GraphTexture, PassParameters};

use crate::lighting::render_sky::RenderSky;
use crate::pass::sky_parameters::SkyParameters;
use crate::scene::level::sky_views::SkyViews;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// What every pass drawing the weather's sky binds as its sky group: the layout [`SkyParameters`] make and the two
/// samplers they read through, and the views a frame's sky takes from the weather textures.
pub struct SkyBindings {
  layout: wgpu::BindGroupLayout,
  clamp: wgpu::Sampler,
  repeat: wgpu::Sampler,
}

impl SkyBindings {
  pub fn new(device: &wgpu::Device) -> Self {
    Self {
      layout: SkyParameters::create_layout(device),
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

  /// A sky's textures, the sun's sprite and the environment cubes as the cache holds them now, each slot its kind's
  /// placeholder until its file is up; the first environment slot, and every one past `environments`, stands for none.
  pub fn collect(
    &self,
    cache: &WeatherTextureCache,
    (sky, sun): (&RenderSky, Option<&str>),
    environments: &[String],
  ) -> SkyViews {
    let view =
      |reference: &Option<String>, kind: WeatherTextureKind| cache.get_view(reference.as_deref(), kind).clone();

    SkyViews {
      cubes: [0, 1].map(|index| view(&sky.textures[index], WeatherTextureKind::Cube)),
      irradiance: [0, 1].map(|index| view(&sky.environments[index], WeatherTextureKind::Cube)),
      clouds: [0, 1].map(|index| view(&sky.clouds.textures[index], WeatherTextureKind::Flat)),
      sun: cache.get_view(sun, WeatherTextureKind::Flat).clone(),
      environments: std::array::from_fn(|slot| {
        let reference: Option<&str> = slot
          .checked_sub(1)
          .and_then(|index| environments.get(index))
          .filter(|reference| !reference.is_empty())
          .map(String::as_str);

        cache.get_view(reference, WeatherTextureKind::Cube).clone()
      }),
    }
  }

  /// Imports a frame's sky views into its graph, as the parameters every pass drawing the sky binds.
  pub fn import<'r>(
    &'r self,
    views: &'r SkyViews,
    graph: &mut FrameGraph<'_>,
    bindings: &mut GraphBindings<'r>,
  ) -> SkyParameters<'r> {
    let mut import = |label: &'static str, view: &'r wgpu::TextureView| bindings.import_view(&mut *graph, label, view);
    let [sky_cube_0, sky_cube_1] =
      [("sky cube 0", &views.cubes[0]), ("sky cube 1", &views.cubes[1])].map(|(label, view)| import(label, view));
    let [sky_environment_0, sky_environment_1] = [
      ("sky irradiance 0", &views.irradiance[0]),
      ("sky irradiance 1", &views.irradiance[1]),
    ]
    .map(|(label, view)| import(label, view));
    let [sky_clouds_0, sky_clouds_1] =
      [("sky clouds 0", &views.clouds[0]), ("sky clouds 1", &views.clouds[1])].map(|(label, view)| import(label, view));
    let sky_sun: GraphTexture = import("sky sun", &views.sun);
    let environments: [GraphTexture; _] =
      std::array::from_fn(|slot| import("environment cube", &views.environments[slot]));

    SkyParameters {
      sky_cube_0,
      sky_cube_1,
      sky_environment_0,
      sky_environment_1,
      sky_clouds_0,
      sky_clouds_1,
      sky_clamp: &self.clamp,
      sky_repeat: &self.repeat,
      environments,
      sky_sun,
    }
  }
}
