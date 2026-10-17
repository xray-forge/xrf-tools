use std::ptr;
use std::sync::{Arc, Weak};

use glam::Vec4;
use xrf_renderer_core::{FrameGraph, GraphBindings, GraphRuntime, GraphTexture, StorageArray};

use crate::frame::view_target_handles::ViewTargetHandles;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::host::render_rain::RenderRain;
use crate::lighting::ambient_gust::AmbientGust;
use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::render_rainfall::RenderRainfall;
use crate::pass::level_passes::LevelPasses;
use crate::pass::rain_parameters::RainParameters;
use crate::pass::rain_uniform::RainUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::over_scene::add_over_scene;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::weather_model_buffers::WeatherModelBuffers;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;
use crate::scene::texture::weather_textures::WeatherTextures;

/// The falling rain, while the weather rains and the view shows it: the splash's model built for the level's weather,
/// the uniform at the rain cover's window, the streak's and the splash's textures, and the drops drawn.
#[derive(Default)]
pub struct FallingRainFeature {
  /// The splash's model, with the level's weather it was built for, held weakly so its address is never another's.
  splash: Option<(Weak<RenderLevelWeather>, WeatherModelBuffers)>,
  uniform: RainUniform,
  /// The streak's and the splash's textures, as the cache holds them when the frame is prepared.
  views: Option<[wgpu::TextureView; 2]>,
  /// The streaks drawn and the splash's indices, none while it does not rain.
  draw: Option<(u32, u32)>,
}

impl FallingRainFeature {
  /// Takes this frame's rain where `rainfall` falls: the splash's model built for the level's weather, the uniform at
  /// the rain cover's window and shape (`cover`), and the textures; answers the uniform, which the wet surfaces follow,
  /// none where the weather has no rain to draw.
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    (lighting, weather): (&RenderLighting, Option<&Arc<RenderLevelWeather>>),
    (rainfall, gust, time): (Option<RenderRainfall>, AmbientGust, f32),
    (cover, (texels, reach)): (Vec4, (f32, f32)),
    weather_textures: WeatherTextures<'_>,
  ) -> Option<RainUniform> {
    self.draw = None;
    self.views = None;

    let rainfall: RenderRainfall = rainfall?;
    let (weather, rain): (&Arc<RenderLevelWeather>, &RenderRain) =
      weather.and_then(|weather| weather.rain.as_ref().map(|rain: &RenderRain| (weather, rain)))?;

    if self
      .splash
      .as_ref()
      .is_none_or(|(built, _)| !ptr::eq(built.as_ptr(), Arc::as_ptr(weather)))
    {
      self.splash = Some((
        Arc::downgrade(weather),
        WeatherModelBuffers::new(device, rain.drop.as_ref()),
      ));
    }

    let (_, splash) = self.splash.as_ref()?;
    let uniform: RainUniform = RainUniform::new(
      &rainfall,
      (lighting.wind, gust.strength),
      (cover, Vec4::new(texels, reach, 0.0, 0.0)),
      time,
      splash.index_count,
    );
    let flat = |reference: Option<&str>| weather_textures.get_view(reference, WeatherTextureKind::Flat).clone();

    self.uniform = uniform;
    self.views = Some([
      flat(Some(&rain.streak)),
      flat(rain.drop.as_ref().map(|drop| drop.texture.as_str())),
    ]);
    self.draw = Some((uniform.count, splash.index_count));

    Some(uniform)
  }

  /// Whether the rain falls this frame.
  pub fn is_raining(&self) -> bool {
    self.draw.is_some()
  }

  /// Follows the rain cover to where it stands once it has moved, as the drops read it.
  pub fn follow_cover(&mut self, cover: Vec4, (texels, reach): (f32, f32)) {
    self.uniform.window = cover;
    self.uniform.cover_shape = Vec4::new(texels, reach, 0.0, 0.0);
  }

  /// Adds the drops and the splashes over the scene, read through the rain `cover`; nothing while it does not rain or
  /// no cover is drawn.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (handles, cover): (ViewTargetHandles, Option<GraphTexture>),
  ) {
    let (Some(counts), Some(cover), Some([streak, splash]), Some((_, model))) =
      (self.draw, cover, &self.views, &self.splash)
    else {
      return;
    };
    let parameters: RainParameters = RainParameters {
      rain: runtime.push_uniform(&self.uniform),
      cover,
      streak_texture: bindings.import_view(&mut *graph, "rain streak", streak),
      splash_texture: bindings.import_view(&mut *graph, "rain splash", splash),
      rain_sampler: passes.rain.get_sampler(),
      splash_vertices: StorageArray::new(bindings.import_buffer(&mut *graph, "splash vertices", &model.vertices)),
      splash_indices: StorageArray::new(bindings.import_buffer(&mut *graph, "splash indices", &model.indices)),
    };

    add_over_scene(&mut *graph, handles, "rain")
      .parameters(&parameters)
      .record(move |context| passes.rain.record(context, view, &parameters, counts));
  }
}

impl ViewFeature for FallingRainFeature {
  /// The splash's model, kept for the level's weather.
  fn get_memory(&self) -> u64 {
    self
      .splash
      .as_ref()
      .map_or(0, |(_, model)| model.vertices.size() + model.indices.size())
  }

  fn release(&mut self) {
    *self = Self::default();
  }
}
