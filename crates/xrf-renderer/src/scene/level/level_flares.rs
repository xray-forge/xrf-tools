use std::sync::Arc;
use std::time::Instant;

use glam::{Vec3, Vec4};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphColorAttachment, GraphRuntime, GraphTexture, StorageValue,
  StorageValueMut, UniformBinding,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_lens_flare::RenderLensFlare;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::flare_draw_parameters::FlareDrawParameters;
use crate::pass::flare_measure_parameters::FlareMeasureParameters;
use crate::pass::flare_pass::{FlarePass, GRADIENT_INSTANCE};
use crate::pass::flare_texture_parameters::FlareTextureParameters;
use crate::pass::flare_uniform::{FLARE_SLOTS, FlareUniform};
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::lens_flare_fade::LensFlareFade;
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// The sun a level's viewport draws as `CLensFlare` does: the lens flare its keyframes name, faded between them, its
/// sprite in the sky and its flares and gradient over the frame, each shown as much as the sun is.
pub struct LevelFlares {
  fade: LensFlareFade,
  /// What the flares draw this frame by.
  values: FlareUniform,
  /// How much of the sun shows, eased on the GPU from frame to frame.
  state: wgpu::Buffer,
  /// Each flare's instance and texture as the weather textures hold it this frame, the gradient's last.
  textures: Vec<(u32, wgpu::TextureView)>,
  is_drawn: bool,
  /// Whether this frame draws the flares as well as the gradient.
  is_flared: bool,
  last: Option<Instant>,
}

impl LevelFlares {
  pub fn new(device: &wgpu::Device) -> Self {
    Self {
      fade: LensFlareFade::new(),
      values: FlareUniform::default(),
      state: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("flare visibility"),
        size: 16,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
      }),
      textures: Vec::new(),
      is_drawn: false,
      is_flared: true,
      last: None,
    }
  }

  /// Asks for the textures of the lens flare drawn now and of the one the keyframes name, so a fade has both.
  pub fn request(
    &self,
    (lighting, weather): (&RenderLighting, Option<&Arc<RenderLevelWeather>>),
    weather_textures: &mut WeatherTextureCache,
    assets: &Arc<dyn RenderAssetSource>,
  ) {
    let Some(weather) = weather else {
      return;
    };
    let names = [lighting.sky.sun.as_deref(), self.fade.get_shown().0];

    for flare in names.into_iter().flatten().filter_map(|name| weather.suns.get(name)) {
      let references = flare
        .sprite
        .iter()
        .map(|it| it.texture.as_str())
        .chain(flare.flares.iter().map(|it| it.texture.as_str()))
        .chain(flare.gradient.iter().map(|it| it.texture.as_str()));

      for reference in references {
        weather_textures.request(reference, WeatherTextureKind::Flat, assets);
      }
    }
  }

  /// Steps the fade and notes what the flares draw by and with; answers the sun's sprite as the sky draws it, its
  /// texture and its colour and radius, none where no sprite is drawn.
  pub fn prepare(
    &mut self,
    (lighting, weather): (&RenderLighting, Option<&Arc<RenderLevelWeather>>),
    options: &RenderViewOptions,
    weather_textures: &WeatherTextureCache,
    (view, rate): (&CameraView, f32),
    has_targets: bool,
  ) -> Option<(String, Vec4)> {
    let now: Instant = Instant::now();
    let delta: f32 = self
      .last
      .map_or(0.0, |last| now.saturating_duration_since(last).as_secs_f32());
    let times = |name: &str| {
      weather
        .and_then(|it| it.suns.get(name))
        .map(|it| (it.rise_time, it.down_time))
    };

    self.last = Some(now);
    self.fade.advance(lighting.sky.sun.as_deref(), times, now, rate);
    self.is_drawn = false;

    let (Some(name), faded) = self.fade.get_shown() else {
      return None;
    };
    let name: String = name.to_owned();
    let flare: &RenderLensFlare = weather.and_then(|it| it.suns.get(&name))?;
    let toward_sun: Vec3 = -lighting.get_sun_direction();
    let to_sun: Vec3 = view.view.transform_vector3(toward_sun).normalize_or_zero();

    if !options.mode.is_lit || !options.show.is_sky_visible || lighting.sun_color.max_element() <= 0.0 {
      return None;
    }

    let light: Vec3 = lighting.sun_color.clamp(Vec3::ZERO, Vec3::ONE);
    let sprite: Option<(String, Vec4)> = flare.sprite.as_ref().map(|sprite| {
      let color: Vec3 = if sprite.is_colorless { Vec3::ONE } else { light };

      (sprite.texture.clone(), (color * faded).extend(sprite.radius))
    });

    self.values = FlareUniform::new(flare, (to_sun, toward_sun), lighting.sun_color, faded, delta);

    self.collect_textures(flare, weather_textures);
    self.is_flared = options.show.is_lens_flared;
    self.is_drawn = has_targets && ((self.is_flared && !flare.flares.is_empty()) || flare.gradient.is_some());

    sprite
  }

  /// The lens flare drawn now, faded in or out, if any.
  pub fn get_shown(&self) -> Option<&str> {
    self.fade.get_shown().0
  }

  /// Measures how much of the sun shows and draws the flares and the gradient, where this frame draws them.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    pass: &'a FlarePass,
    (targets, lit): (ViewTargetHandles, LightingHandles<'a>),
    view: &'a ViewBinding,
  ) {
    if !self.is_drawn {
      return;
    }

    let flares: UniformBinding<FlareUniform> = runtime.push_uniform(&self.values);
    let state: GraphBuffer = bindings.import_buffer(&mut *graph, "flare visibility", &self.state);
    let measure: FlareMeasureParameters = FlareMeasureParameters {
      flares,
      state: StorageValueMut::new(state),
      depth_target: targets.depth,
      shadow_maps: lit.shadow_maps,
      shadows: lit.shadows,
    };
    let draw: FlareDrawParameters = FlareDrawParameters {
      flares,
      state: StorageValue::new(state),
      flare_sampler: pass.get_sampler(),
    };
    let draws: Vec<(u32, FlareTextureParameters)> = self
      .textures
      .iter()
      .filter(|(instance, _)| self.is_flared || *instance == GRADIENT_INSTANCE)
      .map(|(instance, texture)| {
        let flare_texture: GraphTexture = bindings.import_view(&mut *graph, "flare texture", texture);

        (*instance, FlareTextureParameters { flare_texture })
      })
      .collect();

    // How much of the sun shows, measured from the depth, which the draw reads on the GPU: kept, since what it writes
    // is the flares' own.
    graph
      .add_compute_pass("flare visibility")
      .parameters(&measure)
      .keep()
      .record(move |context| pass.record_measure(context, view, &measure));

    let builder = draws.iter().fold(
      graph.add_raster_pass("flares").parameters(&draw),
      |builder, (_, texture)| builder.parameters(texture),
    );

    builder
      .color(GraphColorAttachment::new(targets.scene, wgpu::LoadOp::Load))
      .record(move |context| pass.record_draw(context, view, &draw, &draws));
  }

  /// Each flare's texture and the gradient's, as the weather textures hold them now.
  fn collect_textures(&mut self, flare: &RenderLensFlare, weather_textures: &WeatherTextureCache) {
    let view = |reference: &str| {
      weather_textures
        .get_view(Some(reference), WeatherTextureKind::Flat)
        .clone()
    };
    let flares = flare
      .flares
      .iter()
      .take(FLARE_SLOTS)
      .enumerate()
      .map(|(index, it)| (index as u32, view(&it.texture)));
    let gradient = flare.gradient.iter().map(|it| (GRADIENT_INSTANCE, view(&it.texture)));

    self.textures = flares.chain(gradient).collect();
  }
}
