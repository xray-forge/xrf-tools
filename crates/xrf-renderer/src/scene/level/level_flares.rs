use std::sync::Arc;
use std::time::Instant;

use glam::{Vec3, Vec4};
use xrf_renderer_core::{FrameGraph, GraphColorAttachment, GraphTextureAccess};

use crate::camera::camera_view::CameraView;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_lens_flare::RenderLensFlare;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::flare_pass::{FlarePass, GRADIENT_INSTANCE};
use crate::pass::flare_uniform::{FLARE_SLOTS, FlareUniform};
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::lens_flare_fade::LensFlareFade;
use crate::scene::level::level_shadows::LevelShadows;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// What the flares' texture groups were bound for: the weather textures' generation and the lens flare.
type FlareTexturesKey = (u64, String);

/// The sun a level's viewport draws as `CLensFlare` does: the lens flare its keyframes name, faded between them, its
/// sprite in the sky and its flares and gradient over the frame, each shown as much as the sun is.
pub struct LevelFlares {
  fade: LensFlareFade,
  uniform: wgpu::Buffer,
  /// How much of the sun shows, eased on the GPU from frame to frame.
  state: wgpu::Buffer,
  draw_group: Option<wgpu::BindGroup>,
  /// Keyed by the targets and the shadow maps it reads.
  measure_group: Option<((u64, u64), wgpu::BindGroup)>,
  /// Each flare's instance and texture, keyed by the textures' generation and the lens flare.
  textures: Option<(FlareTexturesKey, Vec<(u32, wgpu::BindGroup)>)>,
  is_drawn: bool,
  /// Whether this frame draws the flares as well as the gradient.
  is_flared: bool,
  last: Option<Instant>,
}

impl LevelFlares {
  pub fn new(device: &wgpu::Device) -> Self {
    Self {
      fade: LensFlareFade::new(),
      uniform: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("flares"),
        size: size_of::<FlareUniform>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      state: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("flare visibility"),
        size: 16,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
      }),
      draw_group: None,
      measure_group: None,
      textures: None,
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

  /// Steps the fade, writes the flares' uniform and binds what they draw with; answers the sun's sprite as the sky
  /// draws it, its texture and its colour and radius, none where no sprite is drawn.
  #[allow(clippy::too_many_arguments)]
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pass: &FlarePass,
    (lighting, weather): (&RenderLighting, Option<&Arc<RenderLevelWeather>>),
    options: &RenderViewOptions,
    weather_textures: &WeatherTextureCache,
    (view, rate): (&CameraView, f32),
    (targets, targets_epoch, shadows): (Option<&ViewTargets>, u64, &LevelShadows),
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

    if !options.is_lit || !options.is_sky_visible || lighting.sun_color.max_element() <= 0.0 {
      return None;
    }

    let light: Vec3 = lighting.sun_color.clamp(Vec3::ZERO, Vec3::ONE);
    let sprite: Option<(String, Vec4)> = flare.sprite.as_ref().map(|sprite| {
      let color: Vec3 = if sprite.is_colorless { Vec3::ONE } else { light };

      (sprite.texture.clone(), (color * faded).extend(sprite.radius))
    });

    queue.write_buffer(
      &self.uniform,
      0,
      bytemuck::bytes_of(&FlareUniform::new(
        flare,
        (to_sun, toward_sun),
        lighting.sun_color,
        faded,
        delta,
      )),
    );

    self.prepare_groups(
      device,
      pass,
      flare,
      (&name, weather_textures),
      (targets, targets_epoch, shadows),
    );
    self.is_flared = options.is_lens_flared;
    self.is_drawn = targets.is_some() && ((self.is_flared && !flare.flares.is_empty()) || flare.gradient.is_some());

    sprite
  }

  /// The lens flare drawn now, faded in or out, if any.
  pub fn get_shown(&self) -> Option<&str> {
    self.fade.get_shown().0
  }

  /// Measures how much of the sun shows and draws the flares and the gradient, where this frame draws them.
  pub fn add_passes<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    pass: &'a FlarePass,
    targets: ViewTargetHandles,
    view: &'a ViewBinding,
  ) {
    let (true, Some(draw_group), Some((_, measure_group)), Some((_, textures))) =
      (self.is_drawn, &self.draw_group, &self.measure_group, &self.textures)
    else {
      return;
    };
    let draws: Vec<(u32, &'a wgpu::BindGroup)> = textures
      .iter()
      .filter(|(instance, _)| self.is_flared || *instance == GRADIENT_INSTANCE)
      .map(|(instance, group)| (*instance, group))
      .collect();

    // How much of the sun shows, measured from the depth, which the draw reads on the GPU: kept, since what it writes
    // is the flares' own.
    graph
      .add_compute_pass("flare visibility")
      .texture(targets.depth, GraphTextureAccess::Sampled)
      .keep()
      .record(move |context| pass.record_measure(context.get_pass(), view, measure_group));
    graph
      .add_raster_pass("flares")
      .color(GraphColorAttachment::new(targets.scene, wgpu::LoadOp::Load))
      .record(move |context| pass.record_draw(context.get_pass(), view, draw_group, &draws));
  }

  fn prepare_groups(
    &mut self,
    device: &wgpu::Device,
    pass: &FlarePass,
    flare: &RenderLensFlare,
    (name, weather_textures): (&str, &WeatherTextureCache),
    (targets, targets_epoch, shadows): (Option<&ViewTargets>, u64, &LevelShadows),
  ) {
    if self.draw_group.is_none() {
      self.draw_group = Some(pass.create_draw_group(device, &self.uniform, &self.state));
    }

    let measure_key: (u64, u64) = (targets_epoch, shadows.get_epoch());

    if self.measure_group.as_ref().is_none_or(|(key, _)| *key != measure_key)
      && let Some(targets) = targets
    {
      self.measure_group = Some((
        measure_key,
        pass.create_measure_group(device, (&self.uniform, &self.state), targets, shadows),
      ));
    }

    let texture_key: FlareTexturesKey = (weather_textures.get_generation(), name.to_owned());

    if self.textures.as_ref().is_none_or(|(key, _)| *key != texture_key) {
      let group = |reference: &str| {
        pass.create_texture_group(
          device,
          weather_textures.get_view(Some(reference), WeatherTextureKind::Flat),
        )
      };
      let flares = flare
        .flares
        .iter()
        .take(FLARE_SLOTS)
        .enumerate()
        .map(|(index, it)| (index as u32, group(&it.texture)));
      let gradient = flare.gradient.iter().map(|it| (GRADIENT_INSTANCE, group(&it.texture)));

      self.textures = Some((texture_key, flares.chain(gradient).collect()));
    }
  }
}
