use std::sync::Arc;
use std::time::Instant;

use glam::Vec3;
use xrf_renderer_core::{
  GraphBuffer, GraphColorAttachment, GraphDeclaration, GraphTexture, StorageValue, StorageValueMut, UniformBinding,
};

use crate::feature::effects::flares::flare_draw_parameters::FlareDrawParameters;
use crate::feature::effects::flares::flare_frame::FlareFrame;
use crate::feature::effects::flares::flare_measure_parameters::FlareMeasureParameters;
use crate::feature::effects::flares::flare_pass::FlarePass;
use crate::feature::effects::flares::flare_texture_parameters::FlareTextureParameters;
use crate::feature::effects::flares::flare_uniform::FlareUniform;
use crate::feature::effects::flares::lens_flare_fade::LensFlareFade;
use crate::feature::effects::flares::sun_sprite::SunSprite;
use crate::feature::view::view_feature::ViewFeature;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::weather::flares::render_lens_flare::RenderLensFlare;
use crate::host::weather::render_level_weather::RenderLevelWeather;
use crate::lighting::weather::render_lighting::RenderLighting;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::level::view::view_prepare::ViewPrepare;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;
use crate::scene::texture::weather_textures::WeatherTextures;

/// The sun a level's viewport draws as `CLensFlare` does: the lens flare its keyframes name, faded between them, its
/// sprite in the sky and its flares and gradient over the frame, each shown as much as the sun is.
pub struct FlaresFeature {
  fade: LensFlareFade,
  /// What the flares draw this frame by.
  values: FlareUniform,
  /// How much of the sun shows, eased on the GPU from frame to frame while the flares draw.
  state: Option<wgpu::Buffer>,
  /// Each flare's instance and texture as the weather textures hold it this frame, the gradient's last.
  textures: Vec<(u32, wgpu::TextureView)>,
  is_drawn: bool,
  /// Whether this frame draws the flares as well as the gradient.
  is_flared: bool,
  last: Option<Instant>,
}

impl FlaresFeature {
  pub fn new() -> Self {
    Self {
      fade: LensFlareFade::new(),
      values: FlareUniform::default(),
      state: None,
      textures: Vec::new(),
      is_drawn: false,
      is_flared: true,
      last: None,
    }
  }

  /// Asks for the textures of the lens flare drawn now and of the one the keyframes name, so a fade has both.
  pub fn request(
    &self,
    lighting: &RenderLighting,
    weather: Option<&Arc<RenderLevelWeather>>,
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

  /// Steps the fade and notes what the flares draw by and with, their visibility made while they draw and let go of
  /// while they do not; answers the sun's sprite as the sky draws it, its texture and its colour and radius, none where
  /// no sprite is drawn.
  pub fn prepare(&mut self, frame: &ViewPrepare<'_>, now: Instant) -> Option<SunSprite> {
    let sprite: Option<SunSprite> = self.prepare_drawn(frame, now);
    let device: &wgpu::Device = frame.device;

    if !self.is_drawn {
      self.state = None;
    } else if self.state.is_none() {
      self.state = Some(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("flare visibility"),
        size: 16,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
      }));
    }

    sprite
  }

  fn prepare_drawn(
    &mut self,
    &ViewPrepare {
      view,
      options,
      lighting,
      weather,
      weather_textures,
      weather_rate: rate,
      ..
    }: &ViewPrepare<'_>,
    now: Instant,
  ) -> Option<SunSprite> {
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
    let sprite: Option<SunSprite> = flare.sprite.as_ref().map(|sprite| {
      let color: Vec3 = if sprite.is_colorless { Vec3::ONE } else { light };

      SunSprite {
        texture: sprite.texture.clone(),
        look: (color * faded).extend(sprite.radius),
      }
    });

    self.values = FlareUniform::new(
      flare,
      FlareFrame {
        to_sun,
        toward_sun,
        color: lighting.sun_color,
        faded,
        delta,
      },
    );

    self.collect_textures(flare, weather_textures);
    self.is_flared = options.show.is_lens_flared;
    self.is_drawn = (self.is_flared && !flare.flares.is_empty()) || flare.gradient.is_some();

    sprite
  }

  /// The lens flare drawn now, faded in or out, if any.
  pub fn get_shown(&self) -> Option<&str> {
    self.fade.get_shown().0
  }

  /// Measures how much of the sun shows and draws the flares and the gradient, where this frame draws them.
  pub fn add_passes<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    ViewRecord {
      passes,
      view,
      handles: targets,
      lit,
      ..
    }: ViewRecord<'a>,
  ) {
    let (true, Some(state)) = (self.is_drawn, &self.state) else {
      return;
    };
    let pass: &FlarePass = &passes.effects.flares;
    let flares: UniformBinding<FlareUniform> = runtime.push_uniform(&self.values);
    let state: GraphBuffer = bindings.import_buffer(&mut *graph, "flare visibility", state);
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
      lighting: lit.lighting,
    };
    let draws: Vec<(u32, FlareTextureParameters)> = self
      .textures
      .iter()
      .filter(|(instance, _)| self.is_flared || *instance == FlarePass::GRADIENT_INSTANCE)
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
  fn collect_textures(&mut self, flare: &RenderLensFlare, weather_textures: WeatherTextures<'_>) {
    let view = |reference: &str| {
      weather_textures
        .get_view(Some(reference), WeatherTextureKind::Flat)
        .clone()
    };
    let flares = flare
      .flares
      .iter()
      .take(FlareUniform::SLOTS)
      .enumerate()
      .map(|(index, it)| (index as u32, view(&it.texture)));
    let gradient = flare
      .gradient
      .iter()
      .map(|it| (FlarePass::GRADIENT_INSTANCE, view(&it.texture)));

    self.textures = flares.chain(gradient).collect();
  }
}

impl ViewFeature for FlaresFeature {
  /// The sun's visibility, eased from frame to frame while the flares draw; the textures are the weather's.
  fn get_memory(&self) -> u64 {
    self.state.as_ref().map_or(0, wgpu::Buffer::size)
  }

  fn release(&mut self) {
    self.state = None;
    self.is_drawn = false;
  }
}
