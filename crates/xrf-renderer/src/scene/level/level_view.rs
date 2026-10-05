use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use glam::{Mat4, Vec2, Vec3, Vec4};
use xrf_engine_target::XrayEngine;
use xrf_error::XrfResult;
use xrf_material::XraySurfaceDraw;
use xrf_renderer_core::{ExecutedGraph, FrameGraph, GraphBindings, GraphCompileOptions, GraphRuntime};

use xrf_math::EPS_S;

use crate::camera::camera_view::CameraView;
use crate::contract::render_ambient_report::RenderAmbientReport;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_applied_report::RenderAppliedReport;
use crate::contract::render_applied_shadows::RenderAppliedShadows;
use crate::contract::render_bloom_settings::RenderBloomSettings;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_level_hit::RenderLevelHit;
use crate::contract::render_level_problems::RenderLevelProblems;
use crate::contract::render_lights_report::RenderLightsReport;
use crate::contract::render_load_durations::RenderLoadDurations;
use crate::contract::render_load_failure::RenderLoadFailure;
use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_model_pose::RenderModelPose;
use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_particles_report::RenderParticlesReport;
use crate::contract::render_pool_use::RenderPoolUse;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_sector_skip::RenderSectorSkip;
use crate::contract::render_selection::RenderSelection;
use crate::contract::render_selection_target::RenderSelectionTarget;
use crate::contract::render_spawn_category::RenderSpawnCategory;
use crate::contract::render_static_report::RenderStaticReport;
use crate::contract::render_surface_geometry::RenderSurfaceGeometry;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_water_settings::RenderWaterSettings;
use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::fsr_targets::FsrTargets;
use crate::frame::pick_target::PickTarget;
use crate::frame::smaa_targets::SmaaTargets;
use crate::frame::smoothing_target::SmoothingTarget;
use crate::frame::temporal_history::TemporalHistory;
use crate::frame::temporal_jitter::TemporalJitter;
use crate::frame::upscale_targets::UpscaleTargets;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::host::render_motion::RenderMotion;
use crate::host::render_rain::RenderRain;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::ambient_occlusion_uniform::AmbientOcclusionUniform;
use crate::pass::bloom_pass::BloomGroups;
use crate::pass::bloom_uniform::BloomUniform;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::fsr_groups::FsrGroups;
use crate::pass::fsr_uniform::FsrUniform;
use crate::pass::grass_pass::GrassPass;
use crate::pass::level_passes::LevelPasses;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::rain_bindings::RainBindings;
use crate::pass::rain_uniform::RainUniform;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_draw_groups::StaticDrawGroups;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;
use crate::pass::temporal_uniform::TemporalUniform;
use crate::pass::thunder_uniform::ThunderUniform;
use crate::pass::upscale_uniform::UpscaleUniform;
use crate::pass::view_binding::ViewBinding;
use crate::pass::view_light_groups::ViewLightGroups;
use crate::pass::wet_uniform::WetUniform;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::ambient_frame::AmbientFrame;
use crate::scene::level::ambient_gust::AmbientGust;
use crate::scene::level::level_flares::LevelFlares;
use crate::scene::level::level_overlays::LevelOverlays;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::level_shadows::LevelShadows;
use crate::scene::level::level_smoothing::LevelSmoothing;
use crate::scene::level::level_water::{LevelWater, WaterFrame};
use crate::scene::level::lights_frame::LightsFrame;
use crate::scene::level::posed_skeleton::PosedSkeleton;
use crate::scene::level::rain_cover::RainCover;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::shadow_sway::ShadowSway;
use crate::scene::level::view_info::ViewInfo;
use crate::scene::level::view_state::ViewState;
use crate::scene::level::weather_model_buffers::WeatherModelBuffers;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::static_scene::static_selection::StaticSelection;
use crate::scene::static_scene::static_slot_info::StaticSlotInfo;
use crate::scene::static_scene::static_sorted_place::StaticSortedPlace;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;
use crate::thread::render_workers::RenderWorkers;

/// What a pick's texel says it met: a cluster, by its index and place, or an impostor, by its index.
const PICKED_CLUSTER: u32 = 1;
const PICKED_IMPOSTOR: u32 = 2;

/// Sectors put on the GPU at most each frame, so a level's open spreads over frames rather than stalling one.
const SECTORS_PER_FRAME: usize = 4;

/// Spawned models put into the scene at most in one frame.
const MODELS_PER_FRAME: usize = 16;

/// What a sky's bind group binds: the weather textures' generation, the references of its six slots, and how many
/// environment cubes.
type SkyGroupKey = (u64, [Option<String>; 7], u64);

/// A level drawn in one viewport: read by its loader, held on the GPU, drawn into the viewport's G-buffer and lit.
pub struct LevelView {
  /// This frame as prepared, which its passes read.
  info: ViewInfo,
  /// What the view keeps from one frame to the next.
  state: ViewState,
  /// The level it draws.
  scene: LevelScene,
  cull_params: wgpu::Buffer,
  occlusion: wgpu::Buffer,
  lighting: wgpu::Buffer,
  /// The cull's and the draws' bind groups, with the scene generation (and the cull, the targets epoch) they bind.
  cull_group: Option<((u64, u64), wgpu::BindGroup)>,
  draw_groups: Option<(u64, StaticDrawGroups)>,
  /// The lighting passes' bind groups, made again with the targets, and the shadow maps' epoch they bind.
  light_groups: Option<(u64, ViewLightGroups)>,
  /// The sky's textures as bound, with the cache's generation and the references they bind.
  sky_group: Option<(SkyGroupKey, wgpu::BindGroup)>,
  /// Bumped whenever the sky's bind group is made again, which the water's follows.
  sky_version: u64,
  /// The water, its uniform, and what its enhanced kind reads and draws first.
  water: LevelWater,
  /// What the present pass shows, a [`PresentUniform`].
  present: wgpu::Buffer,
  temporal_uniform: wgpu::Buffer,
  fsr_uniform: wgpu::Buffer,
  upscale_uniform: wgpu::Buffer,
  /// The present pass's bind group, with the targets' and the upscale's epochs and the frame it shows.
  present_group: Option<((u64, u64, usize), wgpu::BindGroup)>,
  /// The sorted composited clusters' bind group, with the scene generation and the list's epoch it binds.
  sorted_group: Option<((u64, u64), wgpu::BindGroup)>,
  /// The overlay pass's bind group, with the targets' epoch it binds.
  overlay_group: Option<(u64, wgpu::BindGroup)>,
  rain_cover: RainCover,
  rain: wgpu::Buffer,
  /// The rain's bind group, with the weather textures' generation and the splash's weather it binds.
  rain_group: Option<((u64, usize), wgpu::BindGroup)>,
  wet: wgpu::Buffer,
  /// The wet surfaces' bind groups, with the targets' epoch and the weather textures' generation they bind.
  wet_groups: Option<((u64, u64), [wgpu::BindGroup; 2])>,
  thunder: wgpu::Buffer,
  /// A strike's bind groups, with the weather textures' generation, the weather and the bolt they bind.
  thunder_groups: Option<((u64, usize, String), [wgpu::BindGroup; 3])>,
  /// The sun's sprite, lens flares and gradient.
  flares: LevelFlares,
  /// What the bloom's build and its two blurs read, and what they draw with, at the targets' epoch.
  bloom_uniforms: [wgpu::Buffer; 3],
  bloom_groups: Option<(u64, BloomGroups)>,
  shadows: LevelShadows,
  occlusion_uniform: wgpu::Buffer,
}

impl LevelView {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    view_layout: &wgpu::BindGroupLayout,
    source: Arc<dyn RenderLevelSource>,
    workers: &RenderWorkers,
  ) -> Self {
    let uniform = |label: &str, size: usize| -> wgpu::Buffer {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      })
    };

    let scene: LevelScene = LevelScene::new(device, queue, view_layout, source, workers);

    Self {
      info: ViewInfo::default(),
      state: ViewState::new(device, queue),
      rain_cover: RainCover::new(device, view_layout, scene.statics.args.size()),
      scene,
      cull_params: uniform("static cull", size_of::<StaticCullParams>()),
      occlusion: uniform("static occlusion", size_of::<StaticOcclusionUniform>()),
      lighting: uniform("lighting", size_of::<LightingUniform>()),
      cull_group: None,
      draw_groups: None,
      light_groups: None,
      sky_group: None,
      sky_version: 0,
      water: LevelWater::new(device),
      present: uniform("present", size_of::<PresentUniform>()),
      temporal_uniform: uniform("temporal", size_of::<TemporalUniform>()),
      fsr_uniform: uniform("fsr2", size_of::<FsrUniform>()),
      upscale_uniform: uniform("upscale", size_of::<UpscaleUniform>()),
      present_group: None,
      overlay_group: None,
      sorted_group: None,
      rain: uniform("rain", size_of::<RainUniform>()),
      rain_group: None,
      wet: uniform("wet", size_of::<WetUniform>()),
      wet_groups: None,
      thunder: uniform("thunder", size_of::<ThunderUniform>()),
      thunder_groups: None,
      flares: LevelFlares::new(device),
      bloom_uniforms: ["bloom build", "bloom across", "bloom down"]
        .map(|label| uniform(label, size_of::<BloomUniform>())),
      bloom_groups: None,
      shadows: LevelShadows::new(device),
      occlusion_uniform: uniform("ambient occlusion", size_of::<AmbientOcclusionUniform>()),
    }
  }

  /// Puts the sectors and spawned models the loaders finished since the last frame into the scene, a few a frame, and
  /// asks for the textures the lighting's sky draws with.
  #[allow(clippy::too_many_arguments)]
  pub fn load(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    (textures, weather_textures): (&mut TextureCache, &mut WeatherTextureCache),
    grass_pass: &GrassPass,
    (lighting, weather): (&RenderLighting, Option<&Arc<RenderLevelWeather>>),
    options: &RenderViewOptions,
  ) {
    let assets: Arc<dyn RenderAssetSource> = Arc::clone(&self.scene.source) as Arc<dyn RenderAssetSource>;

    if let Some(rain) = weather.and_then(|weather| weather.rain.as_ref())
      && lighting.rain.is_some()
      && options.is_rainy
      && options.is_lit
    {
      weather_textures.request(&rain.streak, WeatherTextureKind::Flat, &assets);

      if let Some(drop) = &rain.drop {
        weather_textures.request(&drop.texture, WeatherTextureKind::Flat, &assets);
      }

      if let Some(wet) = weather.and_then(|weather| weather.wet.as_ref()) {
        weather_textures.request(&wet.splash, WeatherTextureKind::Volume, &assets);
        weather_textures.request(&wet.flow, WeatherTextureKind::Flat, &assets);
      }
    }

    // Every bolt's textures are held while the weather may strike, so a strike has them.
    if let Some(thunder) = weather.and_then(|weather| weather.thunder.as_ref())
      && options.is_thundering
      && options.is_lit
    {
      let references = thunder.models.iter().map(|model| model.mesh.texture.as_str()).chain(
        thunder
          .bolts
          .values()
          .flat_map(|bolt| [bolt.top.texture.as_str(), bolt.center.texture.as_str()]),
      );

      for reference in references.filter(|reference| !reference.is_empty()) {
        weather_textures.request(reference, WeatherTextureKind::Flat, &assets);
      }
    }

    if options.is_lit && options.is_sky_visible {
      self.flares.request((lighting, weather), weather_textures, &assets);
      weather_textures.request_sky(&lighting.sky, options.is_clouded, &assets);
    } else {
      // The irradiance cubes light the hemisphere whether or not the sky is drawn.
      for reference in lighting.sky.environments.iter().flatten() {
        weather_textures.request(reference, WeatherTextureKind::Cube, &assets);
      }
    }

    if self.scene.environments.0 != textures.get_environments_generation() {
      self.scene.environments = (
        textures.get_environments_generation(),
        textures.list_environments().to_vec(),
      );
    }

    // Only the cubes this scene samples are kept loaded; another scene's are bound as placeholders here.
    for slot in &self.scene.statics.environment_slots {
      if let Some(reference) = textures.get_environment(*slot) {
        weather_textures.request(reference, WeatherTextureKind::Cube, &assets);
      }
    }

    self.scene.lights.poll(textures, &assets);
    self.scene.particles.poll(device, textures, &assets);

    if let Some(slots) = self.scene.grass.poll(device, grass_pass, textures, &assets) {
      self.scene.statics.texture_slots.extend(slots);
    }

    for (sector, package) in self.scene.loader.take(SECTORS_PER_FRAME) {
      match package {
        Ok((package, tally)) => {
          self
            .scene
            .skipped
            .extend(package.description.skipped.iter().map(|skip| RenderSectorSkip {
              sector,
              skip: skip.clone(),
            }));

          let started: Instant = Instant::now();

          self.scene.statics.add_sector(
            device,
            queue,
            encoder,
            textures,
            &assets,
            self.scene.source.get_surfaces(),
            &package,
          );
          self.scene.sector_time = started.elapsed().as_secs_f32() * 1000.0;
          self.scene.surfaces.merge(tally);
        }
        Err(reason) => {
          self.scene.failed_sectors.push(RenderLoadFailure {
            name: sector.to_string(),
            reason,
          });
        }
      }
    }

    for (model, places, skeleton) in self.scene.spawn.take(MODELS_PER_FRAME) {
      self
        .scene
        .statics
        .add_model(device, queue, encoder, textures, &assets, &model, &places);

      if let Some(skeleton) = skeleton.filter(|_| model.skin.is_some()) {
        for place in &places {
          self.scene.skeletons.insert(place.object, PosedSkeleton::new(&skeleton));
        }
      }
    }

    self.note_load_durations(textures);
  }

  /// Notes how long the level had been opening when each part of it finished, the first frame it is seen finished.
  fn note_load_durations(&mut self, textures: &TextureCache) {
    // Every part has finished by the time the whole has.
    if self.scene.load_durations.ready.is_some() {
      return;
    }

    let elapsed: Duration = self.scene.started.elapsed();
    let finished: [bool; 6] = [
      self.are_sectors_done(),
      self.scene.spawn.is_done(),
      self.scene.grass.is_loaded(),
      self.scene.lights.is_loaded(),
      self.scene.particles.is_loaded(),
      self.describe_load(textures).is_ready,
    ];
    let RenderLoadDurations {
      sectors,
      spawn,
      grass,
      lights,
      particles,
      ready,
    } = &mut self.scene.load_durations;

    for (duration, is_finished) in [sectors, spawn, grass, lights, particles, ready]
      .into_iter()
      .zip(finished)
    {
      if is_finished {
        duration.get_or_insert(elapsed);
      }
    }
  }

  /// Whether every sector has been taken in or failed.
  fn are_sectors_done(&self) -> bool {
    (self.scene.statics.sectors.len() + self.scene.failed_sectors.len()) as u32 == self.scene.loader.get_total()
  }

  /// How much each shader table entry draws across the sectors resident.
  pub fn measure_surfaces(&self) -> Vec<RenderSurfaceGeometry> {
    self.scene.surfaces.list()
  }

  /// What became of every texture the level's surfaces sample.
  pub fn describe_textures(&self, textures: &TextureCache) -> Vec<RenderTextureReport> {
    textures.describe(&self.scene.statics.texture_slots)
  }

  /// Sizes the targets to the viewport and writes what this frame's cull and lighting read.
  #[allow(clippy::too_many_arguments)]
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    passes: LevelPasses<'_>,
    view: &CameraView,
    ((width, height), output): ((u32, u32), RenderRect),
    field_of_view: f32,
    options: &RenderViewOptions,
    (lighting, weather): (&RenderLighting, Option<&Arc<RenderLevelWeather>>),
    weather_textures: &WeatherTextureCache,
    weather_rate: f32,
  ) {
    if !self.state.targets.as_ref().is_some_and(|it| it.is_sized(width, height)) {
      let targets: ViewTargets = ViewTargets::new(device, width, height);
      let pyramid: DepthPyramid = DepthPyramid::new(device, width, height);
      let groups: Vec<wgpu::BindGroup> = passes.pyramid.create_bind_groups(device, &targets.depth, &pyramid);

      self.state.targets = Some(targets);
      self.state.pyramid = Some((pyramid, groups));
      self.state.targets_epoch += 1;
      self.state.temporal = None;
      self.state.fsr = None;
      // A pyramid of another size holds no depth this frame can be tested against.
      self.state.history = None;
      self.light_groups = None;
    }

    self.scene.statics.reset_draws(device, queue, encoder);
    self.pose_skeletons(queue);

    if self.state.overlays.as_ref().is_some_and(|it| it.skeleton.is_some()) {
      let segments: Vec<(Vec3, Vec3)> = self.list_skeleton_segments();

      if let Some(overlays) = &mut self.state.overlays {
        overlays.set_skeleton(device, &segments);
      }
    }

    let generation: u64 = self.scene.statics.get_generation();
    let cull_key: (u64, u64) = (generation, self.state.targets_epoch);

    if self.cull_group.as_ref().is_none_or(|(it, _)| *it != cull_key)
      && let Some((pyramid, _)) = &self.state.pyramid
    {
      let group: wgpu::BindGroup = passes.cull.create_bind_group(
        device,
        &self.scene.statics,
        &self.cull_params,
        &pyramid.view,
        &self.occlusion,
        (self.scene.statics.lists.get_buffer(), &self.scene.statics.args),
      );

      self.cull_group = Some((cull_key, group));
    }

    if self.draw_groups.as_ref().is_none_or(|(it, _)| *it != generation) {
      self.draw_groups = Some((
        generation,
        passes.gbuffer.create_bind_groups(device, &self.scene.statics),
      ));
    }

    self.shadows.prepare(device, options.shadows.resolution);

    let shadow_epoch: u64 = self.shadows.get_epoch();

    if self
      .light_groups
      .as_ref()
      .is_none_or(|(epoch, _)| *epoch != shadow_epoch)
      && let Some(targets) = &self.state.targets
    {
      let groups: ViewLightGroups = ViewLightGroups {
        sun: passes
          .sun
          .create_bind_group(device, targets, passes.table, &self.lighting, &self.shadows),
        lights: passes.lights.create_bind_groups(
          device,
          targets,
          passes.table,
          &self.scene.lights.get_buffers(),
          self.scene.lights.get_shadow_atlas(),
        ),
        occlusion: passes
          .ambient_occlusion
          .create_bind_groups(device, targets, &self.occlusion_uniform),
        combine: passes.combine.create_bind_group(
          device,
          targets,
          passes.table,
          &self.lighting,
          &self.state.exposure.state,
        ),
        composited: passes.composited.create_bind_group(
          device,
          passes.table,
          (&self.lighting, &self.state.exposure.state),
          &self.shadows,
        ),
        haze: passes
          .sky_haze
          .create_bind_group(device, &self.lighting, &self.state.exposure.state),
        sun_shafts: passes.sun_shafts.create_bind_group(
          device,
          targets,
          &self.shadows,
          (&self.lighting, &self.state.exposure.state),
        ),
        exposure: passes.exposure.create_bind_group(device, targets, &self.state.exposure),
      };

      self.light_groups = Some((shadow_epoch, groups));
    }

    self.state.exposure.prepare(queue, &options.exposure, Instant::now());

    self.info.sun_sprite = self.flares.prepare(
      device,
      queue,
      passes.flares,
      (lighting, weather),
      options,
      weather_textures,
      (view, weather_rate),
      (self.state.targets.as_ref(), self.state.targets_epoch, &self.shadows),
    );

    let sky = &lighting.sky;
    let sky_key: SkyGroupKey = (
      weather_textures.get_generation(),
      [
        sky.textures[0].clone(),
        sky.textures[1].clone(),
        sky.environments[0].clone(),
        sky.environments[1].clone(),
        sky.clouds.textures[0].clone(),
        sky.clouds.textures[1].clone(),
        self.info.sun_sprite.as_ref().map(|(texture, _)| texture.clone()),
      ],
      self.scene.environments.0,
    );

    if self.sky_group.as_ref().is_none_or(|(key, _)| *key != sky_key) {
      self.sky_group = Some((
        sky_key,
        passes.sky.create_bind_group(
          device,
          weather_textures,
          (sky, self.info.sun_sprite.as_ref().map(|(texture, _)| texture.as_str())),
          &self.scene.environments.1,
        ),
      ));
      self.sky_version += 1;
    }

    // The ambient effects blow the wind the grass, the rain and the campfires read this frame.
    self.scene.particles.update_ambient(
      view,
      options,
      weather.map(|level| AmbientFrame {
        ambients: &lighting.ambients,
        level,
      }),
    );

    let gust: AmbientGust = self.scene.particles.get_gust();

    self.water.prepare(
      device,
      queue,
      passes.water,
      options,
      WaterFrame {
        targets: self
          .state
          .targets
          .as_ref()
          .map(|targets| (targets, self.state.targets_epoch)),
        lighting: &self.lighting,
        skies: (
          [
            weather_textures.get_view(sky.textures[0].as_deref(), WeatherTextureKind::Cube),
            weather_textures.get_view(sky.textures[1].as_deref(), WeatherTextureKind::Cube),
          ],
          passes.sky.get_clamp(),
          self.sky_version,
        ),
        intensity: lighting.water_intensity,
        wind: lighting.wind,
        rain: lighting.rain.map_or(0.0, |rain| rain.density),
        time: self.scene.started.elapsed().as_secs_f32(),
      },
    );

    let sway_time: f32 = self.scene.started.elapsed().as_secs_f32();
    let wind: WindUniform = WindUniform::new(lighting.trees.as_ref().filter(|_| options.is_windy), sway_time)
      .following(self.state.last_wind.as_ref());

    self.state.last_wind = Some(wind);

    self.info.sway = (wind.get_amplitude(), sway_time);
    self.prepare_rain(
      device,
      queue,
      passes,
      (lighting, weather),
      (options, gust),
      weather_textures,
    );
    self.prepare_thunder(device, queue, passes, (lighting, weather), options, weather_textures);
    queue.write_buffer(&self.scene.statics.wind, 0, bytemuck::bytes_of(&wind));

    // A keyframe whose textures are not all up is blended out, so a sky still going up shows the other one.
    let side = |index: usize| {
      let clouds: Option<&str> = sky.clouds.textures[index].as_deref().filter(|_| options.is_clouded);

      weather_textures.is_settled(sky.textures[index].as_deref())
        && weather_textures.is_settled(sky.environments[index].as_deref())
        && weather_textures.is_settled(clouds)
    };
    let (is_first_up, is_second_up) = (side(0), side(1));
    let frame: LightingFrame = LightingFrame {
      is_adapting: self.state.exposure.is_adapting(),
      sky_blend: match (is_first_up, is_second_up) {
        (true, false) => 0.0,
        (false, true) => 1.0,
        _ => sky.blend,
      },
      is_irradiance_up: sky
        .environments
        .iter()
        .all(|reference| reference.as_deref().is_some_and(|it| weather_textures.is_up(it))),
      clouds_time: self.scene.started.elapsed().as_secs_f32(),
      sun_sprite: self
        .info
        .sun_sprite
        .as_ref()
        .filter(|(texture, _)| weather_textures.is_up(texture))
        .map_or(Vec4::ZERO, |(_, sprite)| *sprite),
    };

    self.info.is_hazing = options.is_lit && options.is_sky_visible && options.is_sky_hazed;
    self.info.is_shafted = options.is_lit
      && options.is_sun_shafted
      && lighting.get_sun_shafts(&options.sun_shafts) > 0.0
      && options.shadows.get_cascade_count() > 0;
    self.info.is_wallmarked = options.is_wallmarked;
    self.prepare_bloom(device, queue, passes, options, lighting.engine);

    if !options.is_occlusion_culled {
      self.state.history = None;
    }

    // The engine's screen: the viewport's pixels, widened for a lens narrower than its 90 degrees.
    let screen: f32 =
      (width * height) as f32 * (90.0 / field_of_view.max(1.0)).powi(2) * (EPS_S + options.lod.geometry_lod);
    let threshold = |area: f32| -> f32 { (area / 3.0).powi(2) / screen };

    self.info.cull = StaticCullParams {
      cluster_count: self.scene.statics.get_cluster_count(),
      row_count: self.scene.statics.get_row_count(),
      batch_count: StaticBatch::COUNT as u32,
      impostor_count: self.scene.statics.get_impostor_count(),
      glod_start: threshold(options.lod.ssa_glod_start),
      glod_end: threshold(options.lod.ssa_glod_end),
      discard_below: options.lod.ssa_discard.powi(2) / screen,
      candidate_capacity: self.scene.statics.get_list_capacity(),
      is_occluding: options.is_occlusion_culled as u32,
      lod_a: threshold(options.lod.ssa_a),
      lod_b: threshold(options.lod.ssa_b),
      is_impostors: options.lod.is_impostors as u32,
      hidden_groups: to_hidden_groups(options),
      pad: [0; 3],
      lod_origin: view.position.extend(1.0),
    };
    self.info.matrices = (view.view, view.projection);
    self.scene.grass.prepare(
      device,
      queue,
      passes.grass,
      &options.grass,
      view,
      self.info.cull.discard_below,
      (
        self.scene.started.elapsed().as_secs_f32(),
        options.is_windy,
        gust.strength,
      ),
    );
    // The campfires switch and the moving zones move whatever of them is drawn, so their lights and particles follow
    // them alike.
    self.scene.campfires.prepare(options.is_campfire_lit);
    self.scene.object_motions.prepare();
    self.scene.lights.prepare(
      queue,
      LightsFrame {
        camera: view,
        settings: &options.lights,
        lod: (self.info.cull.glod_start, self.info.cull.glod_end),
        contents: self.scene.statics.get_contents(),
        sway: &to_sway(&self.scene.statics, self.info.sway),
        campfires: &mut self.scene.campfires,
        motions: &mut self.scene.object_motions,
      },
    );
    self.info.camera = *view;
    self.info.sun_direction = lighting.get_sun_direction();
    self.info.shadow_settings = options.shadows.clone();
    self.info.ambient_occlusion = options.ambient_occlusion;
    self.info.output = output;
    self.info.upscaling = options.upscaling;
    self.info.debug_view = options.debug_view;
    self.info.is_wireframe = options.is_wireframe;
    self.info.corrections = options.corrections;
    self.info.is_occlusion_drawn = options.is_lit && options.ambient_occlusion.is_enabled;
    self.prepare_temporal(device, queue, passes, view);
    self.prepare_smoothing(device, passes, options.antialiasing);
    self.prepare_upscale(device, queue, passes);
    self.info.lights_settings = options.lights;

    queue.write_buffer(
      &self.occlusion_uniform,
      0,
      bytemuck::bytes_of(&AmbientOcclusionUniform::new(
        &options.ambient_occlusion,
        view.projection,
        (width.div_ceil(2), height.div_ceil(2)),
      )),
    );
    queue.write_buffer(&self.cull_params, 0, bytemuck::bytes_of(&self.info.cull));
    self.prepare_sorted(device, queue, passes, view, self.scene.statics.get_generation());

    if let Some((pyramid, _)) = &self.state.pyramid {
      let (history_view, history_projection): (Mat4, Mat4) = self.state.history.unwrap_or(self.info.matrices);

      queue.write_buffer(
        &self.occlusion,
        0,
        bytemuck::bytes_of(&StaticOcclusionUniform {
          view: history_view,
          projection: history_projection,
          size: Vec2::new(pyramid.width as f32, pyramid.height as f32),
          levels: pyramid.levels,
          has_history: self.state.history.is_some() as u32,
        }),
      );
    }
    queue.write_buffer(
      &self.lighting,
      0,
      bytemuck::bytes_of(&LightingUniform::new(lighting, view.view, options, &frame)),
    );

    self
      .scene
      .particles
      .step(view, options, &mut self.scene.campfires, &mut self.scene.object_motions);

    if let Some(targets) = &self.state.targets {
      self.scene.particles.upload(
        device,
        queue,
        passes.particles,
        &self.lighting,
        (targets, self.state.targets_epoch),
      );
    }

    self.write_present(queue, options);
  }

  /// Writes what the present pass reads, once the frame knows what draws into the distortion target: `def_distort`
  /// while the water or a particle does, nothing otherwise.
  fn write_present(&self, queue: &wgpu::Queue, options: &RenderViewOptions) {
    let Some(targets) = &self.state.targets else {
      return;
    };
    let water: &RenderWaterSettings = &options.water;
    let is_water_distorting: bool = water.is_enabled && water.is_distorted && options.is_lit;
    let is_distorting: bool = !options.is_wireframe && (is_water_distorting || self.scene.particles.is_distorting());

    queue.write_buffer(
      &self.present,
      0,
      bytemuck::bytes_of(&PresentUniform::new(
        self.info.debug_view,
        self.info.is_occlusion_drawn,
        !targets.is_sized(self.info.output.width, self.info.output.height),
        if is_distorting { water.distortion } else { 0.0 },
        self.info.output,
        &self.info.corrections,
        (self.info.selection_color, self.info.is_bloomed),
      )),
    );
  }

  /// Writes this frame's bloom while the view blooms (`phase_bloom`): the build's threshold over the frame's size, and
  /// the blur across and down, down by the frame's height over its width, one-sided on Monolith as Anomaly's
  /// `bloom_filter.ps` reads it; and what the draws bind.
  fn prepare_bloom(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    passes: LevelPasses<'_>,
    options: &RenderViewOptions,
    engine: XrayEngine,
  ) {
    let bloom: &RenderBloomSettings = &options.bloom;

    self.info.is_bloomed = bloom.is_enabled && options.is_lit && !options.is_wireframe && self.state.targets.is_some();

    let Some(targets) = self.state.targets.as_ref().filter(|_| self.info.is_bloomed) else {
      return;
    };

    let size: (u32, u32) = (targets.width, targets.height);
    let aspect: f32 = size.1 as f32 / size.0 as f32;
    let is_one_sided: bool = engine == XrayEngine::Extended;
    let uniforms: [BloomUniform; 3] = [
      BloomUniform::build(size, bloom.threshold),
      BloomUniform::filter(true, (bloom.radius, bloom.strength), aspect, is_one_sided),
      BloomUniform::filter(false, (bloom.radius, bloom.strength), aspect, is_one_sided),
    ];

    for (buffer, uniform) in self.bloom_uniforms.iter().zip(uniforms) {
      queue.write_buffer(buffer, 0, bytemuck::bytes_of(&uniform));
    }

    if self
      .bloom_groups
      .as_ref()
      .is_none_or(|(epoch, _)| *epoch != self.state.targets_epoch)
    {
      self.bloom_groups = Some((
        self.state.targets_epoch,
        passes.bloom.create_bind_groups(device, targets, &self.bloom_uniforms),
      ));
    }
  }

  /// Writes this frame's rain, while the weather rains and the view shows it: the splash's model built for the
  /// level's weather, the uniform at the cover last drawn, and what the draw binds.
  fn prepare_rain(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    passes: LevelPasses<'_>,
    (lighting, weather): (&RenderLighting, Option<&Arc<RenderLevelWeather>>),
    (options, gust): (&RenderViewOptions, AmbientGust),
    weather_textures: &WeatherTextureCache,
  ) {
    self.info.rain_draw = None;

    let Some((key, rain)) = weather.and_then(|weather| {
      weather
        .rain
        .as_ref()
        .map(|rain: &RenderRain| (Arc::as_ptr(weather) as usize, rain))
    }) else {
      return;
    };
    let Some(rainfall) = lighting.rain.filter(|_| options.is_rainy && options.is_lit) else {
      return;
    };

    if self.scene.splash.as_ref().is_none_or(|(built, _)| *built != key) {
      self.scene.splash = Some((key, WeatherModelBuffers::new(device, rain.drop.as_ref())));
    }

    let Some((_, splash)) = &self.scene.splash else {
      return;
    };
    let uniform: RainUniform = RainUniform::new(
      &rainfall,
      (lighting.wind, gust.strength),
      self.rain_cover.get_window(),
      self.scene.started.elapsed().as_secs_f32(),
      splash.index_count,
    );
    let group_key: (u64, usize) = (weather_textures.get_generation(), key);

    queue.write_buffer(&self.rain, 0, bytemuck::bytes_of(&uniform));

    if self.rain_group.as_ref().is_none_or(|(built, _)| *built != group_key) {
      let flat = |reference: Option<&str>| weather_textures.get_view(reference, WeatherTextureKind::Flat);
      let group: wgpu::BindGroup = passes.rain.create_bind_group(
        device,
        &RainBindings {
          uniform: &self.rain,
          cover: &self.rain_cover.depth,
          streak: flat(Some(&rain.streak)),
          splash: flat(rain.drop.as_ref().map(|drop| drop.texture.as_str())),
          vertices: &splash.vertices,
          indices: &splash.indices,
        },
      );

      self.rain_group = Some((group_key, group));
    }

    self.info.rain_draw = Some((uniform.count, splash.index_count));

    let Some(wet) = weather.and_then(|weather| weather.wet.as_ref()) else {
      return;
    };
    let Some(targets) = &self.state.targets else {
      return;
    };
    let wet_key: (u64, u64) = (self.state.targets_epoch, weather_textures.get_generation());

    queue.write_buffer(
      &self.wet,
      0,
      bytemuck::bytes_of(&WetUniform {
        density: rainfall.density.clamp(0.0, 1.0),
        time: uniform.time,
        is_extended: (lighting.engine == XrayEngine::Extended) as u32 as f32,
        pad: 0.0,
        window: uniform.window,
      }),
    );

    if self.wet_groups.as_ref().is_none_or(|(built, _)| *built != wet_key) {
      let groups: [wgpu::BindGroup; 2] = passes.wet.create_bind_groups(
        device,
        targets,
        &self.rain_cover.depth,
        (
          weather_textures.get_view(Some(&wet.splash), WeatherTextureKind::Volume),
          weather_textures.get_view(Some(&wet.flow), WeatherTextureKind::Flat),
        ),
        &self.wet,
      );

      self.wet_groups = Some((wet_key, groups));
    }
  }

  /// Writes this frame's strike, while a bolt strikes and the view shows it: every bolt model built for the level's
  /// weather, the uniform where the strike stands, and what its model and glows bind.
  fn prepare_thunder(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    passes: LevelPasses<'_>,
    (lighting, weather): (&RenderLighting, Option<&Arc<RenderLevelWeather>>),
    options: &RenderViewOptions,
    weather_textures: &WeatherTextureCache,
  ) {
    self.info.thunder_draw = None;

    let (Some(weather), Some(strike)) = (weather, &lighting.thunderbolt) else {
      return;
    };
    let Some(thunder) = weather
      .thunder
      .as_ref()
      .filter(|_| options.is_thundering && options.is_lit)
    else {
      return;
    };
    let Some(bolt) = thunder.bolts.get(&strike.bolt) else {
      return;
    };
    let key: usize = Arc::as_ptr(weather) as usize;

    if self
      .scene
      .thunder_models
      .as_ref()
      .is_none_or(|(built, _)| *built != key)
    {
      let models: Vec<WeatherModelBuffers> = thunder
        .models
        .iter()
        .map(|model| WeatherModelBuffers::new(device, Some(&model.mesh)))
        .collect();

      self.scene.thunder_models = Some((key, models));
    }

    let Some((_, models)) = &self.scene.thunder_models else {
      return;
    };
    let model = bolt
      .model
      .and_then(|index| Some((models.get(index)?, thunder.models.get(index)?)));
    let group_key: (u64, usize, String) = (weather_textures.get_generation(), key, strike.bolt.clone());

    queue.write_buffer(&self.thunder, 0, bytemuck::bytes_of(&ThunderUniform::new(strike)));

    if self
      .thunder_groups
      .as_ref()
      .is_none_or(|(built, _)| *built != group_key)
    {
      let flat = |reference: &str| weather_textures.get_view(Some(reference), WeatherTextureKind::Flat);
      let buffers: &WeatherModelBuffers = model.map_or(&self.scene.no_model, |(buffers, _)| buffers);
      let model_texture: &str = model.map_or("", |(_, model)| model.mesh.texture.as_str());
      let groups: [wgpu::BindGroup; 3] = [
        passes
          .thunder
          .create_bind_group(device, &self.thunder, flat(model_texture), buffers),
        passes
          .thunder
          .create_bind_group(device, &self.thunder, flat(&bolt.top.texture), &self.scene.no_model),
        passes
          .thunder
          .create_bind_group(device, &self.thunder, flat(&bolt.center.texture), &self.scene.no_model),
      ];

      self.thunder_groups = Some((group_key, groups));
    }

    self.info.thunder_draw = Some((
      [
        model.map_or(XraySurfaceDraw::Opaque, |(_, model)| model.draw),
        bolt.top.draw,
        bolt.center.draw,
      ],
      model.map_or(0, |(buffers, _)| buffers.index_count),
    ));
  }

  /// Records the frame as a frame graph of bridge passes, each the pass it was before the graph, and executes it with
  /// the viewport's runtime: culls the scene and draws it into the G-buffer (what last frame's depth does not hide,
  /// then, culling occlusion, what this frame's first draw does not hide of the rest, leaving this frame's depth reduced
  /// for the next), shadows and lights it, draws the water and what blends over it, and resolves the frame. Which
  /// passes run is decided here; each bridge reaches the view through one lock, as they record one after another.
  ///
  /// # Errors
  ///
  /// Returns an error when the graph cannot compile or execute, which a frame of bridges should never meet.
  #[allow(clippy::too_many_arguments)]
  pub fn record(
    &mut self,
    runtime: &mut GraphRuntime,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    passes: LevelPasses<'_>,
    view_layout: &wgpu::BindGroupLayout,
    view: &ViewBinding,
    textures: &TextureCache,
  ) -> XrfResult<Option<ExecutedGraph>> {
    if self.state.targets.is_none()
      || self.state.pyramid.is_none()
      || self.cull_group.is_none()
      || self.draw_groups.is_none()
    {
      return Ok(None);
    }

    let is_occluding: bool = self.info.cull.is_occluding != 0;

    if is_occluding {
      self.state.history = Some(self.info.matrices);
    }

    let is_drawn: bool = !self.info.is_wireframe;
    let is_wallmarked: bool = self.info.is_wallmarked && is_drawn;
    let is_raining: bool = self.info.rain_draw.is_some();
    let is_wet: bool = is_raining && self.wet_groups.is_some();
    let is_lit: bool = self.light_groups.is_some();
    let has_lights: bool = self.scene.lights.get_count() > 0;
    let is_occlusion_ambient: bool = self.info.ambient_occlusion.is_enabled;
    let has_sky: bool = self.sky_group.is_some();
    let is_hazing: bool = has_sky && self.info.is_hazing;
    let is_composited: bool = is_drawn && has_sky;
    let has_particles: bool = is_drawn && self.scene.particles.is_drawing();
    let is_shafted: bool = self.info.is_shafted;
    let is_rain_drawn: bool = is_raining && self.rain_group.is_some();
    let is_thundering: bool = self.info.thunder_draw.is_some() && self.thunder_groups.is_some();
    let is_bloomed: bool = self.info.is_bloomed && self.bloom_groups.is_some();
    let is_smoothed: bool = self.state.smoothing.is_some();
    let resolve: Option<&'static str> = if self.state.fsr.is_some() {
      Some("fsr2")
    } else if self.state.temporal.is_some() {
      Some("temporal")
    } else if self.state.upscale.is_some() {
      Some("upscale")
    } else {
      None
    };
    let is_sharpened: bool = self.state.upscale.is_some() && self.info.upscaling.is_sharpened();
    let is_adapting: bool = self.state.exposure.is_adapting();
    let texture_group: &wgpu::BindGroup = textures.get_bind_group();
    let cell: Mutex<&mut LevelView> = Mutex::new(self);
    let mut graph: FrameGraph<'_> = FrameGraph::new();

    // A bridge pass recording as the frame did before the graph, the view reached through the cell.
    macro_rules! bridge {
      ($name:literal, |$level:ident, $encoder:ident, $marker:ident| $body:block) => {
        graph.add_encoder_pass($name).bridge().record(|context| {
          let mut guard = cell.lock().expect("level view lock");
          let $level: &mut LevelView = &mut guard;
          let ($encoder, $marker) = context.split();
          let _ = &$marker;

          $body
        });
      };
    }

    bridge!("grass planting", |level, encoder, marker| {
      if let Some(targets) = &level.state.targets {
        targets.clear_distortion(encoder);
      }

      level.scene.grass.plant(encoder, passes.grass);
    });
    bridge!("cull", |level, encoder, marker| {
      if let Some((_, cull_group)) = &level.cull_group {
        passes.cull.dispatch_early(encoder, view, cull_group, &level.info.cull);
      }
    });
    bridge!("g-buffer", |level, encoder, marker| {
      if let (Some(targets), Some((_, draw_groups))) = (&level.state.targets, &level.draw_groups) {
        passes.gbuffer.draw(
          encoder,
          targets,
          view,
          draw_groups,
          texture_group,
          &level.scene.statics.args,
          true,
        );
      }
    });

    if is_occluding {
      bridge!("occlusion", |level, encoder, marker| {
        if let (Some(targets), Some((pyramid, pyramid_groups)), Some((_, cull_group)), Some((_, draw_groups))) = (
          &level.state.targets,
          &level.state.pyramid,
          &level.cull_group,
          &level.draw_groups,
        ) {
          passes.pyramid.dispatch(encoder, pyramid, pyramid_groups);
          passes
            .cull
            .dispatch_late(encoder, view, cull_group, &level.scene.statics);
          passes.gbuffer.draw(
            encoder,
            targets,
            view,
            draw_groups,
            texture_group,
            &level.scene.statics.late,
            false,
          );
        }
      });
    }

    bridge!("stats", |level, encoder, marker| {
      level
        .state
        .stats
        .record(encoder, &level.scene.statics.args, StaticScene::STATS_OFFSET);
    });

    if is_drawn {
      bridge!("grass", |level, encoder, marker| {
        if let Some(targets) = &level.state.targets {
          level
            .scene
            .grass
            .draw(encoder, passes.grass, (targets, view), texture_group);
        }
      });
    }

    if is_wallmarked {
      bridge!("wall marks", |level, encoder, marker| {
        if let (Some(targets), Some((_, draw_groups))) = (&level.state.targets, &level.draw_groups) {
          passes.composited.draw_wallmarks(
            encoder,
            targets,
            view,
            draw_groups,
            texture_group,
            &Self::list_draw_args(&level.scene.statics, &level.info.cull),
          );
        }
      });
    }

    bridge!("shadows", |level, encoder, marker| {
      let LevelView {
        scene,
        info,
        cull_params,
        state,
        occlusion,
        shadows,
        rain_cover,
        ..
      } = &mut *level;
      let LevelScene { statics, lights, .. } = scene;
      let Some((pyramid, _)) = state.pyramid.as_ref() else {
        return;
      };
      let frame: ShadowFrame<'_> = ShadowFrame {
        scene: statics,
        camera: &info.camera,
        settings: &info.shadow_settings,
        sun_direction: info.sun_direction,
        sway: to_sway(statics, info.sway),
        cull_params,
        params: &info.cull,
        pyramid: &pyramid.view,
        occlusion,
        targets_epoch: state.targets_epoch,
        textures,
      };

      shadows.record(device, queue, encoder, passes, view_layout, &frame);
      marker.mark(encoder, "sun shadows");

      if is_raining {
        rain_cover.record(device, queue, encoder, passes, &frame);
        marker.mark(encoder, "rain cover");
      }

      lights.record_shadows(device, queue, encoder, passes, &frame);
      marker.mark(encoder, "light shadows");
    });

    // The rain wets the G-buffer before any light is drawn over it.
    if is_wet {
      bridge!("wet", |level, encoder, marker| {
        if let (Some(targets), Some((_, wet_groups))) = (&level.state.targets, &level.wet_groups) {
          passes.wet.draw(encoder, targets, view, wet_groups);
        }
      });
    }

    if is_lit {
      bridge!("sun", |level, encoder, marker| {
        if let (Some(targets), Some((_, groups))) = (&level.state.targets, &level.light_groups) {
          passes.sun.draw(encoder, targets, view, &groups.sun);
        }

        level.scene.lights.clear_overflow(encoder);

        if !has_lights {
          level.scene.lights.record_overflow(encoder);
        }
      });

      if has_lights {
        bridge!("lights", |level, encoder, marker| {
          if let (Some(targets), Some((_, groups))) = (&level.state.targets, &level.light_groups) {
            passes
              .lights
              .draw(encoder, targets, view, &groups.lights, texture_group);
          }

          level.scene.lights.record_overflow(encoder);
        });
      }

      if is_occlusion_ambient {
        bridge!("ambient occlusion", |level, encoder, marker| {
          if let (Some(targets), Some((_, groups))) = (&level.state.targets, &level.light_groups) {
            passes.ambient_occlusion.draw(
              encoder,
              targets,
              view,
              &groups.occlusion,
              level.info.ambient_occlusion.quality,
            );
          }
        });
      }

      if is_hazing {
        bridge!("haze", |level, encoder, marker| {
          if let (Some(targets), Some((_, groups)), Some((_, sky_group))) =
            (&level.state.targets, &level.light_groups, &level.sky_group)
          {
            passes.sky_haze.draw(encoder, targets, &groups.haze, sky_group);
          }
        });
      }

      if has_sky {
        bridge!("combine", |level, encoder, marker| {
          if let (Some(targets), Some((_, groups)), Some((_, sky_group))) =
            (&level.state.targets, &level.light_groups, &level.sky_group)
          {
            passes.combine.draw(encoder, targets, view, &groups.combine, sky_group);
          }
        });
      }

      bridge!("water", |level, encoder, marker| {
        let args: Vec<&wgpu::Buffer> = Self::list_draw_args(&level.scene.statics, &level.info.cull);
        let (Some(targets), Some((_, draw_groups))) = (&level.state.targets, &level.draw_groups) else {
          return;
        };

        // FSR 2's reactive mask is what the water and the blended surfaces change of the frame drawn so far.
        if let Some((fsr, _)) = &level.state.fsr {
          encoder.copy_texture_to_texture(
            targets.scene_texture.as_image_copy(),
            fsr.opaque_texture.as_image_copy(),
            fsr.opaque_texture.size(),
          );
        }

        level.water.record(
          encoder,
          passes.water,
          targets,
          (view, draw_groups, texture_group),
          &args,
          marker,
        );
      });

      if is_composited {
        bridge!("composited", |level, encoder, marker| {
          let args: Vec<&wgpu::Buffer> = Self::list_draw_args(&level.scene.statics, &level.info.cull);

          if let (Some(targets), Some((_, draw_groups)), Some((_, groups)), Some((_, sky_group))) = (
            &level.state.targets,
            &level.draw_groups,
            &level.light_groups,
            &level.sky_group,
          ) {
            passes.composited.draw(
              encoder,
              targets,
              view,
              draw_groups,
              texture_group,
              (&groups.composited, sky_group),
              &args,
              (
                level.sorted_group.as_ref().map(|(_, group)| group),
                level.info.sorted_count,
              ),
            );
          }
        });
      }

      if has_particles {
        bridge!("particles", |level, encoder, marker| {
          if let Some(targets) = &level.state.targets {
            level
              .scene
              .particles
              .record(encoder, passes.particles, targets, view, texture_group);
          }
        });
      }

      if is_shafted {
        bridge!("sun shafts", |level, encoder, marker| {
          if let (Some(targets), Some((_, groups))) = (&level.state.targets, &level.light_groups) {
            passes.sun_shafts.draw(encoder, targets, view, &groups.sun_shafts);
          }
        });
      }

      if is_rain_drawn {
        bridge!("rain", |level, encoder, marker| {
          if let (Some(targets), Some(counts), Some((_, rain_group))) =
            (&level.state.targets, level.info.rain_draw, &level.rain_group)
          {
            passes.rain.draw(encoder, targets, view, rain_group, counts);
          }
        });
      }

      if is_thundering {
        bridge!("thunder", |level, encoder, marker| {
          if let (Some(targets), Some(draws), Some((_, thunder_groups))) =
            (&level.state.targets, level.info.thunder_draw, &level.thunder_groups)
          {
            passes.thunder.draw(encoder, targets, view, thunder_groups, draws);
          }
        });
      }

      bridge!("flares", |level, encoder, marker| {
        if let Some(targets) = &level.state.targets {
          level.flares.record(encoder, passes.flares, targets, view);
        }
      });

      if is_bloomed {
        bridge!("bloom", |level, encoder, marker| {
          if let (Some(targets), Some((_, groups))) = (&level.state.targets, &level.bloom_groups) {
            passes.bloom.draw(encoder, targets, groups);
          }
        });
      }

      if is_smoothed {
        bridge!("smoothing", |level, encoder, marker| {
          let (Some(targets), Some(smoothing)) = (&level.state.targets, &level.state.smoothing) else {
            return;
          };
          let target: &SmoothingTarget = &smoothing.target;

          match (&smoothing.smaa, passes.smaa) {
            (Some(smaa), Some(pass)) => pass.draw(encoder, smaa, &smoothing.groups, &target.view),
            _ => passes.fxaa.draw(encoder, &smoothing.groups[0], &target.view),
          }

          encoder.copy_texture_to_texture(
            target.texture.as_image_copy(),
            targets.scene_texture.as_image_copy(),
            target.texture.size(),
          );
        });
      }

      // The resolved frame goes where the present pass reads it: the upscaled frame, or the scene drawn at its size.
      match resolve {
        Some("fsr2") => {
          bridge!("fsr2", |level, encoder, marker| {
            let ViewState {
              targets, fsr, upscale, ..
            } = &mut level.state;
            let (Some(targets), Some((fsr, groups))) = (targets.as_ref(), fsr.as_mut()) else {
              return;
            };
            let resolved: &wgpu::Texture = upscale
              .as_ref()
              .map_or(&targets.scene_texture, |(upscale, _)| &upscale.textures[0]);
            let history: &wgpu::Texture = &fsr.history_textures[fsr.index];

            passes
              .fsr
              .draw(encoder, fsr, groups, &mut |encoder, name| marker.mark(encoder, name));
            encoder.copy_texture_to_texture(history.as_image_copy(), resolved.as_image_copy(), history.size());
            fsr.swap();
            marker.mark(encoder, "fsr2 output");
          });
        }
        Some("temporal") => {
          bridge!("temporal", |level, encoder, marker| {
            let ViewState {
              targets,
              temporal,
              upscale,
              ..
            } = &mut level.state;
            let (Some(targets), Some((history, groups))) = (targets.as_ref(), temporal.as_mut()) else {
              return;
            };
            let index: usize = history.index;
            let resolved: &wgpu::Texture = upscale
              .as_ref()
              .map_or(&targets.scene_texture, |(upscale, _)| &upscale.textures[0]);

            passes
              .temporal
              .draw(encoder, view, &groups[index], &history.views[index]);
            encoder.copy_texture_to_texture(
              history.textures[index].as_image_copy(),
              resolved.as_image_copy(),
              history.textures[index].size(),
            );
            history.swap();
          });
        }
        Some(_) => {
          bridge!("upscale", |level, encoder, marker| {
            if let Some((upscale, groups)) = &level.state.upscale {
              passes.upscale.draw_easu(encoder, upscale, &groups[0]);
            }
          });
        }
        None => {}
      }

      if is_sharpened {
        bridge!("sharpen", |level, encoder, marker| {
          if let Some((upscale, groups)) = &level.state.upscale {
            passes.upscale.draw_rcas(encoder, upscale, &groups[1]);
          }
        });
      }

      if is_adapting {
        bridge!("exposure", |level, encoder, marker| {
          if let Some((_, groups)) = &level.light_groups {
            passes.exposure.dispatch(encoder, &groups.exposure);
          }
        });
      }
    }

    graph
      .compile(&GraphCompileOptions::default())?
      .execute(device, runtime, &GraphBindings::new())
      .map(Some)
  }

  /// The draw arguments a forward pass replays: the early phase's, and the late phase's where occlusion culls.
  fn list_draw_args<'s>(scene: &'s StaticScene, params: &StaticCullParams) -> Vec<&'s wgpu::Buffer> {
    if params.is_occluding != 0 {
      vec![&scene.args, &scene.late]
    } else {
      vec![&scene.args]
    }
  }

  /// Draws the frame's visible clusters into a pick's texel, through the frame's camera narrowed to it.
  #[allow(clippy::too_many_arguments)]
  pub fn record_pick(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    draw: &StaticGBufferPass,
    view_layout: &wgpu::BindGroupLayout,
    textures: &TextureCache,
    camera: &CameraUniform,
  ) {
    let Some((_, draw_groups)) = &self.draw_groups else {
      return;
    };
    let target: &PickTarget = self.state.pick_target.get_or_insert_with(|| PickTarget::new(device));
    let view: &ViewBinding = self
      .state
      .pick_view
      .get_or_insert_with(|| ViewBinding::new(device, view_layout));

    view.write(queue, camera);
    draw.pick(
      encoder,
      target,
      view,
      draw_groups,
      textures.get_bind_group(),
      &[&self.scene.statics.args, &self.scene.statics.late],
    );
  }

  /// Reads a recorded pick back, once its frame was submitted, and names what it met.
  pub fn resolve_pick(
    &self,
    device: &wgpu::Device,
    unproject: impl Fn(f32) -> Vec3,
  ) -> XrfResult<Option<RenderLevelHit>> {
    let Some(target) = &self.state.pick_target else {
      return Ok(None);
    };
    let [kind, cluster, place, depth] = target.read(device)?;
    let point: [f32; 3] = unproject(f32::from_bits(depth)).to_array();

    match kind {
      PICKED_CLUSTER => {}
      PICKED_IMPOSTOR => {
        return Ok(
          self
            .scene
            .statics
            .resolve_impostor_pick(cluster)
            .map(|(sector, shader_id)| RenderLevelHit::Surface {
              sector,
              shader_id: shader_id as u32,
              mesh: None,
              place: None,
              is_impostor: true,
              point,
            }),
        );
      }
      _ => return Ok(None),
    }

    let Some((info, instance)) = self.scene.statics.resolve_pick(cluster, place) else {
      return Ok(None);
    };

    if info.sector == StaticSlotInfo::NO_SECTOR {
      return Ok(
        self
          .scene
          .statics
          .resolve_spawn_pick(place)
          .map(|object| RenderLevelHit::Spawn { object, point }),
      );
    }

    Ok(Some(RenderLevelHit::Surface {
      sector: info.sector,
      shader_id: info.shader_id as u32,
      mesh: info.mesh,
      place: instance,
      is_impostor: false,
      point,
    }))
  }

  /// Where the next frame's samples sit within their pixels, in drawn pixels, `y` down: jittered while it resolves
  /// frames temporally and shows the finished frame, still otherwise.
  pub fn next_jitter(&mut self, options: &RenderViewOptions, ratio: f32) -> Vec2 {
    self.info.is_temporal = options.antialiasing.is_temporal() && options.debug_view == RenderDebugView::Final;
    self.info.is_fsr = self.info.is_temporal && options.antialiasing == RenderAntialiasing::Fsr2;
    self.info.jitter_phases = TemporalJitter::get_phases(ratio);
    self.info.jitter = if self.info.is_temporal {
      self.state.jitter.next(ratio)
    } else {
      Vec2::ZERO
    };

    self.info.jitter
  }

  /// This frame's unjittered view projection and the last one's, which the surfaces' motion is measured between; the
  /// same twice for a first frame.
  pub fn next_motion(&mut self, current: Mat4) -> (Mat4, Mat4) {
    (current, self.state.motion_previous.replace(current).unwrap_or(current))
  }

  /// Makes the temporal resolve's histories while it resolves, dropping them otherwise, and writes what it reads: this
  /// frame's view projection, the last one's, and the jitter.
  fn prepare_temporal(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    passes: LevelPasses<'_>,
    view: &CameraView,
  ) {
    let Some(targets) = self
      .state
      .targets
      .as_ref()
      .filter(|_| self.info.is_temporal && !self.info.is_fsr)
    else {
      self.state.temporal = None;
      self.state.temporal_previous = None;
      self.prepare_fsr(device, queue, passes, view);

      return;
    };

    self.state.fsr = None;
    let (width, height): (u32, u32) = (self.info.output.width.max(1), self.info.output.height.max(1));

    if self
      .state
      .temporal
      .as_ref()
      .is_some_and(|(history, _)| !history.is_sized(width, height))
    {
      self.state.temporal = None;
    }

    let (history, _) = self.state.temporal.get_or_insert_with(|| {
      let history: TemporalHistory = TemporalHistory::new(device, width, height);
      let groups: [wgpu::BindGroup; 2] =
        passes
          .temporal
          .create_bind_groups(device, targets, &history, &self.temporal_uniform);

      (history, groups)
    });
    let current: Mat4 = view.get_view_projection();
    let (previous, previous_view): (Mat4, Mat4) = self.state.temporal_previous.unwrap_or((current, view.view));

    queue.write_buffer(
      &self.temporal_uniform,
      0,
      bytemuck::bytes_of(&TemporalUniform::new(
        current,
        previous,
        previous_view,
        self.info.jitter,
        history.is_valid && self.state.temporal_previous.is_some(),
      )),
    );
    self.state.temporal_previous = Some((current, view.view));
  }

  /// Makes FSR 2's targets while it resolves, dropping them otherwise, and writes its constants.
  fn prepare_fsr(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, passes: LevelPasses<'_>, view: &CameraView) {
    let Some(targets) = self.state.targets.as_ref().filter(|_| self.info.is_fsr) else {
      self.state.fsr = None;

      return;
    };
    let render: (u32, u32) = (targets.width, targets.height);
    let display: (u32, u32) = (self.info.output.width.max(1), self.info.output.height.max(1));

    if self
      .state
      .fsr
      .as_ref()
      .is_some_and(|(fsr, _)| !fsr.is_sized(render, display))
    {
      self.state.fsr = None;
    }

    let (fsr, _) = self.state.fsr.get_or_insert_with(|| {
      let fsr: FsrTargets = FsrTargets::new(device, render, display, ViewTargets::SCENE);
      let groups: FsrGroups = passes.fsr.create_bind_groups(device, targets, &fsr, &self.fsr_uniform);

      (fsr, groups)
    });

    queue.write_buffer(
      &self.fsr_uniform,
      0,
      bytemuck::bytes_of(&FsrUniform::new(
        (render, display),
        self.info.jitter,
        view,
        (FsrTargets::get_luma_mip_size(render), self.info.jitter_phases),
        fsr.frame_index,
      )),
    );
  }

  /// Lists the models' composited clusters in view, their places back to front by distance and each place's clusters in
  /// their parts' order, as the engine draws its sorted blended objects; binds the list for the composited pass.
  fn prepare_sorted(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    passes: LevelPasses<'_>,
    view: &CameraView,
    generation: u64,
  ) {
    let planes: [Vec4; 6] = view.get_planes();
    let hidden: u32 = self.info.cull.hidden_groups;
    let mut places: Vec<(f32, &StaticSortedPlace)> = self
      .scene
      .statics
      .sorted_places
      .iter()
      .filter(|place| place.group == 0 || hidden & (1 << (place.group - 1)) == 0)
      .filter(|place| place.is_in_view(&planes))
      .map(|place| (place.get_distance(view.position), place))
      .collect();

    places.sort_by(|a, b| b.0.total_cmp(&a.0));

    let entries: Vec<[u32; 2]> = places
      .iter()
      .flat_map(|(_, place)| {
        place
          .clusters
          .iter()
          .flat_map(|(first, count)| (*first..first + count).map(|cluster| [cluster, place.place]))
      })
      .collect();
    let bytes: &[u8] = bytemuck::cast_slice(&entries);

    self.info.sorted_count = entries.len() as u32;

    if bytes.is_empty() {
      return;
    }

    if self
      .state
      .sorted_list
      .as_ref()
      .is_none_or(|buffer| buffer.size() < bytes.len() as u64)
    {
      self.state.sorted_list = Some(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sorted composited"),
        size: (bytes.len() as u64).next_power_of_two(),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }));
      self.state.sorted_epoch += 1;
    }

    let Some(buffer) = &self.state.sorted_list else {
      return;
    };

    queue.write_buffer(buffer, 0, bytes);

    let key: (u64, u64) = (generation, self.state.sorted_epoch);

    if self.sorted_group.as_ref().is_none_or(|(it, _)| *it != key) {
      let [_, _, model] = passes.gbuffer.create_layout_groups(device, &self.scene.statics, buffer);

      self.sorted_group = Some((key, model));
    }
  }

  /// Makes the smoothing pass's targets while one smooths the frame as drawn, dropping them otherwise.
  fn prepare_smoothing(&mut self, device: &wgpu::Device, passes: LevelPasses<'_>, mode: RenderAntialiasing) {
    // SMAA whose lookup textures could not be read smooths as FXAA does, which needs none.
    let mode: RenderAntialiasing = match (mode, passes.smaa) {
      (RenderAntialiasing::Smaa, None) => RenderAntialiasing::Fxaa,
      _ => mode,
    };
    let is_smoothed: bool = matches!(mode, RenderAntialiasing::Fxaa | RenderAntialiasing::Smaa);
    let Some(targets) = self
      .state
      .targets
      .as_ref()
      .filter(|_| is_smoothed && self.info.debug_view == RenderDebugView::Final)
    else {
      self.state.smoothing = None;

      return;
    };

    if self
      .state
      .smoothing
      .as_ref()
      .is_none_or(|it| it.epoch != self.state.targets_epoch || it.mode != mode)
    {
      let target: SmoothingTarget = SmoothingTarget::new(device, targets.width, targets.height, ViewTargets::SCENE);
      let (smaa, groups): (Option<SmaaTargets>, Vec<wgpu::BindGroup>) = match (mode, passes.smaa) {
        (RenderAntialiasing::Smaa, Some(pass)) => {
          let smaa: SmaaTargets = SmaaTargets::new(device, targets.width, targets.height);
          let groups: [wgpu::BindGroup; 3] = pass.create_bind_groups(device, targets, &smaa);

          (Some(smaa), groups.into())
        }
        _ => (None, vec![passes.fxaa.create_bind_group(device, targets)]),
      };

      self.state.smoothing = Some(LevelSmoothing {
        epoch: self.state.targets_epoch,
        mode,
        target,
        smaa,
        groups,
      });
    }
  }

  /// Makes the frame at the viewport's size while the scene is drawn smaller, dropping it otherwise; writes what the
  /// upscale pass reads, and binds the frame the present pass shows.
  fn prepare_upscale(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, passes: LevelPasses<'_>) {
    let Some(targets) = &self.state.targets else {
      return;
    };
    let output: RenderRect = self.info.output;
    let is_upscaled: bool = !targets.is_sized(output.width, output.height);

    if !is_upscaled {
      self.state.upscale = None;
    } else if self
      .state
      .upscale
      .as_ref()
      .is_none_or(|(upscale, _)| !upscale.is_sized(output.width, output.height))
    {
      let upscale: UpscaleTargets = UpscaleTargets::new(device, output.width, output.height);
      let groups: [wgpu::BindGroup; 2] = [
        passes
          .upscale
          .create_bind_group(device, &targets.scene, &self.upscale_uniform),
        passes
          .upscale
          .create_bind_group(device, &upscale.views[0], &self.upscale_uniform),
      ];

      self.state.upscale = Some((upscale, groups));
      self.state.upscale_epoch += 1;
    }

    // An upscale made before the targets were reads a scene since dropped.
    if let Some((upscale, groups)) = &mut self.state.upscale
      && self
        .present_group
        .as_ref()
        .is_none_or(|(key, _)| key.0 != self.state.targets_epoch)
    {
      groups[0] = passes
        .upscale
        .create_bind_group(device, &targets.scene, &self.upscale_uniform);
      groups[1] = passes
        .upscale
        .create_bind_group(device, &upscale.views[0], &self.upscale_uniform);
    }

    queue.write_buffer(
      &self.upscale_uniform,
      0,
      bytemuck::bytes_of(&UpscaleUniform {
        output_size: [output.width as f32, output.height as f32],
        sharpness: self.info.upscaling.get_sharpness(),
        pad: 0.0,
      }),
    );
    if self
      .overlay_group
      .as_ref()
      .is_none_or(|(epoch, _)| *epoch != self.state.targets_epoch)
    {
      let group: wgpu::BindGroup = passes
        .overlay
        .create_bind_group(device, targets, &self.present, &self.lighting);

      self.overlay_group = Some((self.state.targets_epoch, group));
    }

    let shown: usize = usize::from(self.info.upscaling.is_sharpened());
    let key: (u64, u64, usize) = (
      self.state.targets_epoch,
      if is_upscaled { self.state.upscale_epoch } else { 0 },
      shown,
    );

    if self.present_group.as_ref().is_none_or(|(it, _)| *it != key) {
      let upscaled: Option<&wgpu::TextureView> = self.state.upscale.as_ref().map(|(upscale, _)| &upscale.views[shown]);
      let group: wgpu::BindGroup = passes
        .present
        .create_bind_group(device, targets, &self.present, upscaled);

      self.present_group = Some((key, group));
    }
  }

  /// Asks for the counts recorded with the frame just submitted.
  pub fn request_stats(&self) {
    self.state.stats.request();
    self.scene.lights.request_report();
  }

  /// Stands every skinned object as asked from the next frame on.
  pub fn set_model_pose(&mut self, pose: &RenderModelPose) {
    if self.scene.model_pose != *pose {
      self.scene.model_pose = pose.clone();
    }
  }

  /// Writes every skinned object's bone matrices for this frame, and the last frame's beside them; a motion still on
  /// its way poses the bind pose meanwhile.
  fn pose_skeletons(&mut self, queue: &wgpu::Queue) {
    if self.scene.skeletons.is_empty() {
      return;
    }

    let pose: &RenderModelPose = &self.scene.model_pose;
    let motion: Option<&RenderMotion> = match &pose.motion {
      Some(name) => self.scene.motions.get(&self.scene.source, name),
      None => None,
    };

    for (object, skeleton) in &mut self.scene.skeletons {
      let (current, previous) = skeleton.pose(motion, pose.frame, &pose.hidden_bones);

      self.scene.statics.write_pose(queue, *object, &current, &previous);
    }
  }

  /// Every skinned object's bones as segments in renderer space, child then parent, where this frame poses them.
  pub fn list_skeleton_segments(&self) -> Vec<(Vec3, Vec3)> {
    self
      .scene
      .skeletons
      .iter()
      .flat_map(|(object, skeleton)| {
        let place: Mat4 = self
          .scene
          .statics
          .get_object_transform(*object)
          .unwrap_or(Mat4::IDENTITY);

        skeleton
          .list_segments()
          .into_iter()
          .map(move |(child, parent)| (place.transform_point3(child), place.transform_point3(parent)))
      })
      .collect()
  }

  /// What the level could not draw the way it asked, so far.
  pub fn describe_problems(&self) -> RenderLevelProblems {
    RenderLevelProblems {
      skipped: self.scene.skipped.clone(),
      sectors: self.scene.failed_sectors.clone(),
      models: self.scene.spawn.list_failures(),
    }
  }

  /// A spawned object's bounding sphere in renderer space, once its model is in the scene.
  pub fn get_object_sphere(&self, object: u32) -> Option<Vec4> {
    self.scene.statics.get_object_sphere(object)
  }

  /// Plays a weather ambient effect on the next frame, without waiting.
  pub fn play_ambient_now(&mut self) {
    self.scene.particles.play_ambient_now();
  }

  /// Where the weather's ambient effects near the camera stand, none until the particles are read.
  pub fn get_ambient_report(&self) -> Option<RenderAmbientReport> {
    self.scene.particles.get_ambient_report()
  }

  /// What the level's particle systems came to since the last report.
  pub fn take_particles_report(&mut self) -> RenderParticlesReport {
    self.scene.particles.take_report()
  }

  /// Milliseconds the last sector taken in took to put into the scene.
  pub fn get_sector_time(&self) -> f32 {
    self.scene.sector_time
  }

  /// What its frames are drawn with, as resolved from what `options` asked; the weather's light is the viewport's to add.
  pub fn describe_applied(&self, options: &RenderViewOptions) -> RenderAppliedReport {
    let antialiasing: RenderAntialiasing = if self.info.is_temporal {
      options.antialiasing
    } else {
      self
        .state
        .smoothing
        .as_ref()
        .map_or(RenderAntialiasing::None, |it| it.mode)
    };
    let cascades: usize = self.info.shadow_settings.get_cascade_count();

    RenderAppliedReport {
      antialiasing,
      render_scale: self.info.upscaling.scale,
      shadows: (cascades > 0).then(|| RenderAppliedShadows {
        cascades: self.info.shadow_settings.cascades[..cascades].to_vec(),
        resolution: self.shadows.get_maps().resolution,
        filter: self.info.shadow_settings.filter,
      }),
      ambient_occlusion: self
        .info
        .is_occlusion_drawn
        .then_some(self.info.ambient_occlusion.quality),
      lights: self
        .info
        .lights_settings
        .is_enabled
        .then_some(self.info.lights_settings),
      grass: self.scene.grass.get_applied(&options.grass),
      is_water: options.water.is_enabled,
      environment: None,
      sun: self
        .info
        .sun_sprite
        .as_ref()
        .and_then(|_| self.flares.get_shown())
        .map(str::to_owned),
    }
  }

  /// The static draws' pools and what the latest counted frame's cull kept and hid, and its lights.
  pub fn take_stats(&mut self) -> (RenderStaticReport, RenderLightsReport) {
    let [kept_clusters, kept_triangles, occluded_clusters, occluded_triangles] = self.state.stats.take();
    let pools: RenderStaticReport = self.scene.statics.get_pools();
    let phases: u32 = if self.info.cull.is_occluding != 0 { 2 } else { 1 };

    (
      RenderStaticReport {
        surface_list: RenderPoolUse {
          used: kept_clusters,
          ..pools.surface_list
        },
        commands: StaticBatch::list_deferred().count() as u32 * phases + u32::from(self.info.cull.is_impostors != 0),
        kept_clusters,
        kept_triangles,
        occluded_clusters,
        occluded_triangles,
        ..pools
      },
      self.scene.lights.take_report(),
    )
  }

  /// Makes what it draws over its frame where the page's overlays or the selection's box changed: the page's, then a
  /// selected spawned object's box.
  pub fn set_overlays(
    &mut self,
    device: &wgpu::Device,
    overlays: &[RenderOverlay],
    version: u64,
    selection: Option<&RenderSelection>,
  ) {
    let boxed: Option<RenderOverlay> = selection.and_then(|selection| match selection.target {
      RenderSelectionTarget::Spawn { object } => self
        .scene
        .statics
        .get_object_box(object)
        .map(|(transform, corners)| to_box_lines(transform, corners, selection.color)),
      RenderSelectionTarget::Surface { .. } => None,
    });

    if self.state.overlays.as_ref().is_none_or(|it| it.version != version) || self.state.overlays_box != boxed {
      let all: Vec<RenderOverlay> = overlays.iter().cloned().chain(boxed.clone()).collect();

      self.state.overlays = Some(LevelOverlays::new(device, &all, version));
      self.state.overlays_box = boxed;
    }
  }

  /// What a selection marks in the scene now, resolved again only once the target or the scene changed; none where
  /// nothing is selected or what it names is not in the scene yet.
  pub fn resolve_selection(&mut self, selection: Option<&RenderSelection>) -> Option<&StaticSelection> {
    self.info.selection_color = None;

    let selection: &RenderSelection = selection?;
    let generation: u64 = self.scene.statics.get_generation();

    if self
      .state
      .selection
      .as_ref()
      .is_none_or(|(target, at, _)| *target != selection.target || *at != generation)
    {
      self.state.selection = Some((
        selection.target,
        generation,
        self.scene.statics.resolve_selection(&selection.target),
      ));
    }

    let resolved: &StaticSelection = self.state.selection.as_ref()?.2.as_ref()?;

    self.info.selection_color = Some(selection.color);

    Some(resolved)
  }

  /// What it draws over its frame, with the bind group drawing it, once both are made.
  pub fn get_overlays(&self) -> Option<(&wgpu::BindGroup, &LevelOverlays)> {
    Some((&self.overlay_group.as_ref()?.1, self.state.overlays.as_ref()?))
  }

  /// The size its scene is rendered at, once its targets are made.
  pub fn get_render_size(&self) -> Option<(u32, u32)> {
    self.state.targets.as_ref().map(|it| (it.width, it.height))
  }

  /// What puts the level's finished scene into the window, once its targets are made.
  pub fn get_present_group(&self) -> Option<&wgpu::BindGroup> {
    self.present_group.as_ref().map(|(_, group)| group)
  }

  /// Every environment slot it samples, so the cubes no scene samples can be freed.
  pub fn list_environment_slots(&self) -> impl Iterator<Item = u32> + '_ {
    self.scene.statics.environment_slots.iter().copied()
  }

  /// Every texture slot it samples, so the slots no scene samples can be freed.
  pub fn list_texture_slots(&self) -> impl Iterator<Item = u32> + '_ {
    self
      .scene
      .statics
      .texture_slots
      .iter()
      .copied()
      .chain(self.scene.lights.get_projectors().iter().copied())
      .chain(self.scene.particles.get_texture_slots().iter().copied())
  }

  /// Bytes its scene's growing buffers hold on the GPU.
  pub fn get_buffer_bytes(&self) -> u64 {
    self.scene.statics.get_buffer_bytes()
  }

  /// Whether it draws this source.
  pub fn is_showing(&self, source: &Arc<dyn RenderLevelSource>) -> bool {
    Arc::ptr_eq(&self.scene.source, source)
  }

  /// Whether everything it opens with is resident, so it draws as it will.
  pub fn is_ready(&self, textures: &TextureCache) -> bool {
    self.describe_load(textures).is_ready
  }

  /// How far the level has loaded, when that changed since it was last asked.
  pub fn take_report(&mut self, textures: &TextureCache) -> Option<RenderLoadReport> {
    let report: RenderLoadReport = self.describe_load(textures);

    if self.scene.reported == Some(report) {
      return None;
    }

    self.scene.reported = Some(report);

    Some(report)
  }

  /// How far the level has loaded: its sectors taken in or failed, its spawn, its grass, lights and particles read, and
  /// every texture it samples settled; and how long each took.
  pub fn describe_load(&self, textures: &TextureCache) -> RenderLoadReport {
    let settled: u32 = textures.count_settled(self.list_texture_slots());
    let total: u32 = self.list_texture_slots().count() as u32;
    let is_read: bool = self.scene.spawn.is_done()
      && self.scene.grass.is_loaded()
      && self.scene.lights.is_loaded()
      && self.scene.particles.is_loaded();

    RenderLoadReport {
      sectors: self.scene.statics.sectors.len() as u32,
      sectors_total: self.scene.loader.get_total(),
      bytes: self.scene.statics.get_bytes(),
      textures: settled,
      textures_total: total,
      is_ready: self.are_sectors_done() && is_read && settled == total,
      durations: self.scene.load_durations,
    }
  }
}

/// The sway the shadows see this frame: the wind, the trees' reach, and where they stand.
fn to_sway(scene: &StaticScene, (amplitude, time): (f32, f32)) -> ShadowSway<'_> {
  ShadowSway {
    amplitude,
    reach: scene.get_sway_reach(),
    time,
    places: scene.list_swaying(),
  }
}

/// The visibility groups a view hides: each category it does not draw, and the released objects of every one while it
/// draws no released objects.
fn to_hidden_groups(options: &RenderViewOptions) -> u32 {
  [
    RenderSpawnCategory::Props,
    RenderSpawnCategory::Items,
    RenderSpawnCategory::Weapons,
    RenderSpawnCategory::Lamps,
  ]
  .into_iter()
  .fold(0, |hidden, category| {
    let is_shown: bool = options.is_spawned(category);
    let kept: u32 = if is_shown { 0 } else { 1 << (category.get_group() - 1) };
    let released: u32 = if is_shown && options.is_spawned_released {
      0
    } else {
      1 << (category.get_released_group() - 1)
    };

    hidden | kept | released
  })
}

/// A box's twelve edges as lines, its corners where `transform` stands them, in one colour seen through what is in front.
fn to_box_lines(transform: Mat4, [min, max]: [Vec3; 2], color: [f32; 3]) -> RenderOverlay {
  let corner = |index: usize| -> Vec3 {
    transform.transform_point3(Vec3::new(
      if index & 1 == 0 { min.x } else { max.x },
      if index & 2 == 0 { min.y } else { max.y },
      if index & 4 == 0 { min.z } else { max.z },
    ))
  };
  let edges: [(usize, usize); 12] = [
    (0, 1),
    (2, 3),
    (4, 5),
    (6, 7),
    (0, 2),
    (1, 3),
    (4, 6),
    (5, 7),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
  ];
  let positions: Vec<f32> = edges
    .iter()
    .flat_map(|(from, to)| [corner(*from), corner(*to)])
    .flat_map(|point| point.to_array())
    .collect();

  RenderOverlay::Lines {
    colors: color.repeat(positions.len() / 3),
    positions,
    is_depth_tested: false,
  }
}
