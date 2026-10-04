use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use glam::{Mat4, Vec2, Vec3, Vec4};
use xrf_engine_target::XrayEngine;
use xrf_error::XrfResult;
use xrf_material::XraySurfaceDraw;

use xrf_math::EPS_S;

use crate::camera::camera_view::CameraView;
use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_applied_report::RenderAppliedReport;
use crate::contract::render_applied_shadows::RenderAppliedShadows;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_image_corrections::RenderImageCorrections;
use crate::contract::render_level_hit::RenderLevelHit;
use crate::contract::render_level_problems::RenderLevelProblems;
use crate::contract::render_lights_report::RenderLightsReport;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::contract::render_load_durations::RenderLoadDurations;
use crate::contract::render_load_failure::RenderLoadFailure;
use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_model_pose::RenderModelPose;
use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_particles_report::RenderParticlesReport;
use crate::contract::render_pass_cost::RenderPassCost;
use crate::contract::render_pool_use::RenderPoolUse;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_sector_skip::RenderSectorSkip;
use crate::contract::render_selection::RenderSelection;
use crate::contract::render_selection_target::RenderSelectionTarget;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::contract::render_spawn_category::RenderSpawnCategory;
use crate::contract::render_static_report::RenderStaticReport;
use crate::contract::render_surface_geometry::RenderSurfaceGeometry;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_upscaling_settings::RenderUpscalingSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_water_settings::RenderWaterSettings;
use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::fsr_targets::FsrTargets;
use crate::frame::pass_timer::PassTimer;
use crate::frame::pick_target::PickTarget;
use crate::frame::smaa_targets::SmaaTargets;
use crate::frame::smoothing_target::SmoothingTarget;
use crate::frame::stats_readback::StatsReadback;
use crate::frame::temporal_history::TemporalHistory;
use crate::frame::temporal_jitter::TemporalJitter;
use crate::frame::upscale_targets::UpscaleTargets;
use crate::frame::view_exposure::ViewExposure;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::host::render_motion::RenderMotion;
use crate::host::render_rain::RenderRain;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::ambient_occlusion_uniform::AmbientOcclusionUniform;
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
use crate::pass::water_uniform::WaterUniform;
use crate::pass::wet_uniform::WetUniform;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::level_campfires::LevelCampfires;
use crate::scene::level::level_flares::LevelFlares;
use crate::scene::level::level_grass::LevelGrass;
use crate::scene::level::level_lights::LevelLights;
use crate::scene::level::level_loader::LevelLoader;
use crate::scene::level::level_overlays::LevelOverlays;
use crate::scene::level::level_particles::LevelParticles;
use crate::scene::level::level_shadows::LevelShadows;
use crate::scene::level::level_smoothing::LevelSmoothing;
use crate::scene::level::lights_frame::LightsFrame;
use crate::scene::level::model_motions::ModelMotions;
use crate::scene::level::posed_skeleton::PosedSkeleton;
use crate::scene::level::rain_cover::RainCover;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::shadow_sway::ShadowSway;
use crate::scene::level::spawn_loader::SpawnLoader;
use crate::scene::level::surface_tally::SurfaceTally;
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
  source: Arc<dyn RenderLevelSource>,
  loader: LevelLoader,
  spawn: SpawnLoader,
  grass: LevelGrass,
  scene: StaticScene,
  targets: Option<ViewTargets>,
  /// The depth pyramid over the targets, and its reduction's bind group a level.
  pyramid: Option<(DepthPyramid, Vec<wgpu::BindGroup>)>,
  /// Made again with the targets, which the cull's bind group follows.
  targets_epoch: u64,
  cull_params: wgpu::Buffer,
  occlusion: wgpu::Buffer,
  lighting: wgpu::Buffer,
  /// The view and projection the pyramid holds a frame's depth through, once one was reduced.
  history: Option<(Mat4, Mat4)>,
  /// This frame's view and projection, which become the history once its pyramid is reduced.
  frame_view: (Mat4, Mat4),
  /// This frame's camera and shadow settings, which the shadow is fitted and drawn by, and where its sunlight travels.
  frame_camera: CameraView,
  frame_sun: Vec3,
  /// The wind's amplitude this frame and the time the trees sway by: what has still shadows drawn again.
  frame_sway: (f32, f32),
  shadow_settings: RenderShadowSettings,
  /// The cull's and the draws' bind groups, with the scene generation (and the cull, the targets epoch) they bind.
  cull_group: Option<((u64, u64), wgpu::BindGroup)>,
  draw_groups: Option<(u64, StaticDrawGroups)>,
  /// The lighting passes' bind groups, made again with the targets, and the shadow maps' epoch they bind.
  light_groups: Option<(u64, ViewLightGroups)>,
  /// The cubes the scene's environment-mapped models mix toward, by environment slot from the second, as of the cache's
  /// generation of them.
  environments: (u64, Vec<String>),
  /// The sky's textures as bound, with the cache's generation and the references they bind.
  sky_group: Option<(SkyGroupKey, wgpu::BindGroup)>,
  /// Whether this frame blurs the sky into the haze map the distance fades into.
  is_hazing: bool,
  /// Whether this frame lays the wall marks into the albedo.
  is_wallmarked: bool,
  /// Bumped whenever the sky's bind group is made again, which the water's follows.
  sky_version: u64,
  water: wgpu::Buffer,
  /// What the present pass shows, a [`PresentUniform`].
  present: wgpu::Buffer,
  /// Where this frame's samples sit within their pixels, and whether a temporal resolve gathers them.
  jitter: TemporalJitter,
  frame_jitter: Vec2,
  is_temporal: bool,
  /// The temporal resolve's histories and its bind group writing each, while it resolves.
  temporal: Option<(TemporalHistory, [wgpu::BindGroup; 2])>,
  temporal_uniform: wgpu::Buffer,
  /// The last resolved frame's view projection without its jitter, and its view.
  temporal_previous: Option<(Mat4, Mat4)>,
  /// Whether FSR 2 resolves the frames rather than TAA, and how many places this frame's jitter cycles through.
  is_fsr: bool,
  frame_phases: u32,
  /// FSR 2's targets and bind groups, while it resolves.
  fsr: Option<(FsrTargets, FsrGroups)>,
  fsr_uniform: wgpu::Buffer,
  /// The last frame's unjittered view projection, which every surface's motion is measured from.
  motion_previous: Option<Mat4>,
  /// The trees' sway the last frame drew with.
  last_wind: Option<WindUniform>,
  /// The viewport's rectangle in its window, which the frame is upscaled to where it is drawn smaller.
  output: RenderRect,
  upscaling: RenderUpscalingSettings,
  /// The frame at the viewport's size while it is drawn smaller, with its epoch, and the upscale passes' bind groups:
  /// EASU reading the scene, RCAS reading the upscaled frame.
  upscale: Option<(UpscaleTargets, [wgpu::BindGroup; 2])>,
  upscale_epoch: u64,
  upscale_uniform: wgpu::Buffer,
  /// The present pass's bind group, with the targets' and the upscale's epochs and the frame it shows.
  present_group: Option<((u64, u64, usize), wgpu::BindGroup)>,
  /// The models' composited clusters this frame, back to front, as `(cluster, place)` entries, with the bind group
  /// drawing them, the scene generation and the buffer it binds, and how many entries the frame holds.
  sorted_list: Option<wgpu::Buffer>,
  sorted_group: Option<((u64, u64), wgpu::BindGroup)>,
  sorted_epoch: u64,
  sorted_count: u32,
  /// What it draws over its frame, and the overlay pass's bind group with the targets' epoch it binds.
  overlays: Option<LevelOverlays>,
  /// The selection box the overlays were made with, so a box that moved or came into the scene makes them again.
  overlays_box: Option<RenderOverlay>,
  /// What the selection marks, for the target it was resolved for and the scene generation it was resolved in.
  selection: Option<(RenderSelectionTarget, u64, Option<StaticSelection>)>,
  /// The colour the selection is outlined in this frame, or none while nothing it names is drawn.
  selection_color: Option<[f32; 3]>,
  overlay_group: Option<(u64, wgpu::BindGroup)>,
  /// Whether the static surfaces draw as their edges, which nothing composited or planted is drawn over.
  is_wireframe: bool,
  /// The smoothing pass while one smooths the scene as drawn.
  smoothing: Option<LevelSmoothing>,
  /// What corrects this frame's finished image.
  frame_corrections: RenderImageCorrections,
  /// What this frame's present shows, and whether its occlusion was searched.
  frame_debug_view: RenderDebugView,
  frame_occlusion: bool,
  /// The water's bind group, with the sky's version and the targets' epoch it binds.
  water_group: Option<((u64, u64), wgpu::BindGroup)>,
  water_settings: RenderWaterSettings,
  rain_cover: RainCover,
  rain: wgpu::Buffer,
  /// The splash's model, with the level's weather it was built for.
  splash: Option<(usize, WeatherModelBuffers)>,
  /// The rain's bind group, with the weather textures' generation and the splash's weather it binds.
  rain_group: Option<((u64, usize), wgpu::BindGroup)>,
  /// This frame's rain: the streaks drawn and the splash's indices, none while it does not rain.
  rain_draw: Option<(u32, u32)>,
  wet: wgpu::Buffer,
  /// The wet surfaces' bind groups, with the targets' epoch and the weather textures' generation they bind.
  wet_groups: Option<((u64, u64), [wgpu::BindGroup; 2])>,
  thunder: wgpu::Buffer,
  /// Every bolt model of the level's weather, with the weather they were built for, and an empty one the glows bind.
  thunder_models: Option<(usize, Vec<WeatherModelBuffers>)>,
  no_model: WeatherModelBuffers,
  /// A strike's bind groups, with the weather textures' generation, the weather and the bolt they bind.
  thunder_groups: Option<((u64, usize, String), [wgpu::BindGroup; 3])>,
  /// This frame's strike: how its model and glows composite and the model's indices, none while none strikes.
  thunder_draw: Option<([XraySurfaceDraw; 3], u32)>,
  /// The sun's sprite, lens flares and gradient.
  flares: LevelFlares,
  /// Whether this frame adds the sun's light shafts, drawn through its shadow's cascades.
  is_shafted: bool,
  /// The sun's sprite as this frame's sky draws it: its texture, and its colour and radius.
  frame_sun_sprite: Option<(String, Vec4)>,
  /// When the level began opening, which the clouds drift from and its load is timed from.
  started: Instant,
  shadows: LevelShadows,
  lights: LevelLights,
  campfires: LevelCampfires,
  lights_settings: RenderLightsSettings,
  particles: LevelParticles,
  occlusion_uniform: wgpu::Buffer,
  ambient_occlusion: RenderAmbientOcclusionSettings,
  exposure: ViewExposure,
  params: StaticCullParams,
  stats: StatsReadback,
  /// What each pass of its frames cost on the GPU, while timed.
  timer: PassTimer,
  pick_target: Option<PickTarget>,
  pick_view: Option<ViewBinding>,
  surfaces: SurfaceTally,
  /// Each skinned object's skeleton, by its index, the motions they are posed by, and the pose asked for.
  skeletons: HashMap<u32, PosedSkeleton>,
  motions: ModelMotions,
  model_pose: RenderModelPose,
  /// The sectors that could not be read, and the drawables the packer left out of those that were.
  failed_sectors: Vec<RenderLoadFailure>,
  skipped: Vec<RenderSectorSkip>,
  /// Milliseconds the last sector taken in took to put into the scene.
  sector_time: f32,
  /// How long the level had been opening when each part of it finished.
  load_durations: RenderLoadDurations,
  reported: Option<RenderLoadReport>,
}

impl LevelView {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    view_layout: &wgpu::BindGroupLayout,
    source: Arc<dyn RenderLevelSource>,
    workers: &RenderWorkers,
  ) -> Self {
    // Taken before anything is made, so the load is timed from the moment the level began opening.
    let started: Instant = Instant::now();
    let uniform = |label: &str, size: usize| -> wgpu::Buffer {
      device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: size as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      })
    };

    let scene: StaticScene = StaticScene::new(device, queue);

    Self {
      loader: LevelLoader::start(Arc::clone(&source), workers),
      spawn: SpawnLoader::start(Arc::clone(&source), workers),
      grass: LevelGrass::new(device, &source, workers),
      lights: LevelLights::new(device, view_layout, scene.args.size(), &source, workers),
      campfires: LevelCampfires::new(),
      particles: LevelParticles::new(device, &source, workers),
      rain_cover: RainCover::new(device, view_layout, scene.args.size()),
      scene,
      targets: None,
      pyramid: None,
      targets_epoch: 0,
      cull_params: uniform("static cull", size_of::<StaticCullParams>()),
      occlusion: uniform("static occlusion", size_of::<StaticOcclusionUniform>()),
      lighting: uniform("lighting", size_of::<LightingUniform>()),
      history: None,
      frame_view: (Mat4::IDENTITY, Mat4::IDENTITY),
      frame_camera: CameraView {
        position: Vec3::ZERO,
        view: Mat4::IDENTITY,
        projection: Mat4::IDENTITY,
      },
      frame_sun: RenderLighting::default().get_sun_direction(),
      frame_sway: (0.0, 0.0),
      shadow_settings: RenderShadowSettings::default(),
      cull_group: None,
      draw_groups: None,
      light_groups: None,
      environments: (0, Vec::new()),
      sky_group: None,
      is_hazing: false,
      is_wallmarked: true,
      sky_version: 0,
      water: uniform("water", size_of::<WaterUniform>()),
      present: uniform("present", size_of::<PresentUniform>()),
      jitter: TemporalJitter::default(),
      frame_jitter: Vec2::ZERO,
      is_temporal: false,
      temporal: None,
      temporal_uniform: uniform("temporal", size_of::<TemporalUniform>()),
      temporal_previous: None,
      is_fsr: false,
      frame_phases: 1,
      fsr: None,
      fsr_uniform: uniform("fsr2", size_of::<FsrUniform>()),
      motion_previous: None,
      last_wind: None,
      output: RenderRect::default(),
      upscaling: RenderUpscalingSettings::default(),
      upscale: None,
      upscale_epoch: 0,
      upscale_uniform: uniform("upscale", size_of::<UpscaleUniform>()),
      present_group: None,
      smoothing: None,
      is_wireframe: false,
      overlays: None,
      overlays_box: None,
      selection: None,
      selection_color: None,
      overlay_group: None,
      sorted_list: None,
      sorted_group: None,
      sorted_epoch: 0,
      sorted_count: 0,
      frame_corrections: RenderImageCorrections::default(),
      frame_debug_view: RenderDebugView::Final,
      frame_occlusion: false,
      water_group: None,
      water_settings: RenderWaterSettings::default(),
      rain: uniform("rain", size_of::<RainUniform>()),
      splash: None,
      rain_group: None,
      rain_draw: None,
      wet: uniform("wet", size_of::<WetUniform>()),
      wet_groups: None,
      thunder: uniform("thunder", size_of::<ThunderUniform>()),
      thunder_models: None,
      no_model: WeatherModelBuffers::new(device, None),
      thunder_groups: None,
      thunder_draw: None,
      flares: LevelFlares::new(device),
      is_shafted: false,
      frame_sun_sprite: None,
      started,
      shadows: LevelShadows::new(device),
      lights_settings: RenderLightsSettings::default(),
      occlusion_uniform: uniform("ambient occlusion", size_of::<AmbientOcclusionUniform>()),
      ambient_occlusion: RenderAmbientOcclusionSettings::default(),
      exposure: ViewExposure::new(device, queue),
      params: StaticCullParams::default(),
      stats: StatsReadback::new(device),
      timer: PassTimer::new(device, queue),
      pick_target: None,
      pick_view: None,
      surfaces: SurfaceTally::default(),
      skeletons: HashMap::new(),
      motions: ModelMotions::new(workers),
      model_pose: RenderModelPose::default(),
      failed_sectors: Vec::new(),
      skipped: Vec::new(),
      sector_time: 0.0,
      load_durations: RenderLoadDurations::default(),
      reported: None,
      source,
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
    let assets: Arc<dyn RenderAssetSource> = Arc::clone(&self.source) as Arc<dyn RenderAssetSource>;

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

    if self.environments.0 != textures.get_environments_generation() {
      self.environments = (
        textures.get_environments_generation(),
        textures.list_environments().to_vec(),
      );
    }

    // Only the cubes this scene samples are kept loaded; another scene's are bound as placeholders here.
    for slot in &self.scene.environment_slots {
      if let Some(reference) = textures.get_environment(*slot) {
        weather_textures.request(reference, WeatherTextureKind::Cube, &assets);
      }
    }

    self.lights.poll(textures, &assets);
    self.particles.poll(device, textures, &assets);

    if let Some(slots) = self.grass.poll(device, grass_pass, textures, &assets) {
      self.scene.texture_slots.extend(slots);
    }

    for (sector, package) in self.loader.take(SECTORS_PER_FRAME) {
      match package {
        Ok((package, tally)) => {
          self
            .skipped
            .extend(package.description.skipped.iter().map(|skip| RenderSectorSkip {
              sector,
              skip: skip.clone(),
            }));

          let started: Instant = Instant::now();

          self.scene.add_sector(
            device,
            queue,
            encoder,
            textures,
            &assets,
            self.source.get_surfaces(),
            &package,
          );
          self.sector_time = started.elapsed().as_secs_f32() * 1000.0;
          self.surfaces.merge(tally);
        }
        Err(reason) => {
          self.failed_sectors.push(RenderLoadFailure {
            name: sector.to_string(),
            reason,
          });
        }
      }
    }

    for (model, places, skeleton) in self.spawn.take(MODELS_PER_FRAME) {
      self
        .scene
        .add_model(device, queue, encoder, textures, &assets, &model, &places);

      if let Some(skeleton) = skeleton.filter(|_| model.skin.is_some()) {
        for place in &places {
          self.skeletons.insert(place.object, PosedSkeleton::new(&skeleton));
        }
      }
    }

    self.note_load_durations(textures);
  }

  /// Notes how long the level had been opening when each part of it finished, the first frame it is seen finished.
  fn note_load_durations(&mut self, textures: &TextureCache) {
    // Every part has finished by the time the whole has.
    if self.load_durations.ready.is_some() {
      return;
    }

    let elapsed: Duration = self.started.elapsed();
    let finished: [bool; 6] = [
      self.are_sectors_done(),
      self.spawn.is_done(),
      self.grass.is_loaded(),
      self.lights.is_loaded(),
      self.particles.is_loaded(),
      self.describe_load(textures).is_ready,
    ];
    let RenderLoadDurations {
      sectors,
      spawn,
      grass,
      lights,
      particles,
      ready,
    } = &mut self.load_durations;

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
    (self.scene.sectors.len() + self.failed_sectors.len()) as u32 == self.loader.get_total()
  }

  /// How much each shader table entry draws across the sectors resident.
  pub fn measure_surfaces(&self) -> Vec<RenderSurfaceGeometry> {
    self.surfaces.list()
  }

  /// What became of every texture the level's surfaces sample.
  pub fn describe_textures(&self, textures: &TextureCache) -> Vec<RenderTextureReport> {
    textures.describe(&self.scene.texture_slots)
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
    if !self.targets.as_ref().is_some_and(|it| it.is_sized(width, height)) {
      let targets: ViewTargets = ViewTargets::new(device, width, height);
      let pyramid: DepthPyramid = DepthPyramid::new(device, width, height);
      let groups: Vec<wgpu::BindGroup> = passes.pyramid.create_bind_groups(device, &targets.depth, &pyramid);

      self.targets = Some(targets);
      self.pyramid = Some((pyramid, groups));
      self.targets_epoch += 1;
      self.temporal = None;
      self.fsr = None;
      // A pyramid of another size holds no depth this frame can be tested against.
      self.history = None;
      self.light_groups = None;
    }

    self.scene.reset_draws(device, queue, encoder);
    self.pose_skeletons(queue);

    if self.overlays.as_ref().is_some_and(|it| it.skeleton.is_some()) {
      let segments: Vec<(Vec3, Vec3)> = self.list_skeleton_segments();

      if let Some(overlays) = &mut self.overlays {
        overlays.set_skeleton(device, &segments);
      }
    }

    let generation: u64 = self.scene.get_generation();
    let cull_key: (u64, u64) = (generation, self.targets_epoch);

    if self.cull_group.as_ref().is_none_or(|(it, _)| *it != cull_key)
      && let Some((pyramid, _)) = &self.pyramid
    {
      let group: wgpu::BindGroup = passes.cull.create_bind_group(
        device,
        &self.scene,
        &self.cull_params,
        &pyramid.view,
        &self.occlusion,
        (self.scene.lists.get_buffer(), &self.scene.args),
      );

      self.cull_group = Some((cull_key, group));
    }

    if self.draw_groups.as_ref().is_none_or(|(it, _)| *it != generation) {
      self.draw_groups = Some((generation, passes.gbuffer.create_bind_groups(device, &self.scene)));
    }

    self.shadows.prepare(device, options.shadows.resolution);

    let shadow_epoch: u64 = self.shadows.get_epoch();

    if self
      .light_groups
      .as_ref()
      .is_none_or(|(epoch, _)| *epoch != shadow_epoch)
      && let Some(targets) = &self.targets
    {
      let groups: ViewLightGroups = ViewLightGroups {
        sun: passes
          .sun
          .create_bind_group(device, targets, passes.table, &self.lighting, &self.shadows),
        lights: passes.lights.create_bind_groups(
          device,
          targets,
          passes.table,
          &self.lights.get_buffers(),
          self.lights.get_shadow_atlas(),
        ),
        occlusion: passes
          .ambient_occlusion
          .create_bind_groups(device, targets, &self.occlusion_uniform),
        combine: passes
          .combine
          .create_bind_group(device, targets, passes.table, &self.lighting, &self.exposure.state),
        composited: passes.composited.create_bind_group(
          device,
          passes.table,
          (&self.lighting, &self.exposure.state),
          &self.shadows,
        ),
        haze: passes
          .sky_haze
          .create_bind_group(device, &self.lighting, &self.exposure.state),
        sun_shafts: passes.sun_shafts.create_bind_group(
          device,
          targets,
          &self.shadows,
          (&self.lighting, &self.exposure.state),
        ),
        exposure: passes.exposure.create_bind_group(device, targets, &self.exposure),
      };

      self.light_groups = Some((shadow_epoch, groups));
    }

    self.exposure.prepare(queue, &options.exposure, Instant::now());

    self.frame_sun_sprite = self.flares.prepare(
      device,
      queue,
      passes.flares,
      (lighting, weather),
      options,
      weather_textures,
      (view, weather_rate),
      (self.targets.as_ref(), self.targets_epoch, &self.shadows),
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
        self.frame_sun_sprite.as_ref().map(|(texture, _)| texture.clone()),
      ],
      self.environments.0,
    );

    if self.sky_group.as_ref().is_none_or(|(key, _)| *key != sky_key) {
      self.sky_group = Some((
        sky_key,
        passes.sky.create_bind_group(
          device,
          weather_textures,
          (sky, self.frame_sun_sprite.as_ref().map(|(texture, _)| texture.as_str())),
          &self.environments.1,
        ),
      ));
      self.sky_version += 1;
    }

    let water_key: (u64, u64) = (self.sky_version, self.targets_epoch);

    if self.water_group.as_ref().is_none_or(|(key, _)| *key != water_key)
      && let Some(targets) = &self.targets
    {
      let skies = [
        weather_textures.get_view(sky.textures[0].as_deref(), WeatherTextureKind::Cube),
        weather_textures.get_view(sky.textures[1].as_deref(), WeatherTextureKind::Cube),
      ];
      let group: wgpu::BindGroup = passes.water.create_bind_group(
        device,
        targets,
        &self.lighting,
        &self.water,
        skies,
        passes.sky.get_clamp(),
      );

      self.water_group = Some((water_key, group));
    }

    let sway_time: f32 = self.started.elapsed().as_secs_f32();
    let wind: WindUniform = WindUniform::new(lighting.trees.as_ref().filter(|_| options.is_windy), sway_time)
      .following(self.last_wind.as_ref());

    self.last_wind = Some(wind);

    self.frame_sway = (wind.get_amplitude(), sway_time);
    self.prepare_rain(device, queue, passes, (lighting, weather), options, weather_textures);
    self.prepare_thunder(device, queue, passes, (lighting, weather), options, weather_textures);
    queue.write_buffer(&self.scene.wind, 0, bytemuck::bytes_of(&wind));
    self.water_settings = options.water;
    queue.write_buffer(
      &self.water,
      0,
      bytemuck::bytes_of(&WaterUniform::new(
        &options.water,
        lighting.water_intensity,
        self.started.elapsed().as_secs_f32(),
      )),
    );

    // A keyframe whose textures are not all up is blended out, so a sky still going up shows the other one.
    let side = |index: usize| {
      let clouds: Option<&str> = sky.clouds.textures[index].as_deref().filter(|_| options.is_clouded);

      weather_textures.is_settled(sky.textures[index].as_deref())
        && weather_textures.is_settled(sky.environments[index].as_deref())
        && weather_textures.is_settled(clouds)
    };
    let (is_first_up, is_second_up) = (side(0), side(1));
    let frame: LightingFrame = LightingFrame {
      is_adapting: self.exposure.is_adapting(),
      sky_blend: match (is_first_up, is_second_up) {
        (true, false) => 0.0,
        (false, true) => 1.0,
        _ => sky.blend,
      },
      is_irradiance_up: sky
        .environments
        .iter()
        .all(|reference| reference.as_deref().is_some_and(|it| weather_textures.is_up(it))),
      clouds_time: self.started.elapsed().as_secs_f32(),
      sun_sprite: self
        .frame_sun_sprite
        .as_ref()
        .filter(|(texture, _)| weather_textures.is_up(texture))
        .map_or(Vec4::ZERO, |(_, sprite)| *sprite),
    };

    self.is_hazing = options.is_lit && options.is_sky_visible && options.is_sky_hazed;
    self.is_shafted = options.is_lit
      && options.is_sun_shafted
      && lighting.get_sun_shafts(&options.sun_shafts) > 0.0
      && options.shadows.get_cascade_count() > 0;
    self.is_wallmarked = options.is_wallmarked;

    if !options.is_occlusion_culled {
      self.history = None;
    }

    // The engine's screen: the viewport's pixels, widened for a lens narrower than its 90 degrees.
    let screen: f32 =
      (width * height) as f32 * (90.0 / field_of_view.max(1.0)).powi(2) * (EPS_S + options.lod.geometry_lod);
    let threshold = |area: f32| -> f32 { (area / 3.0).powi(2) / screen };

    self.params = StaticCullParams {
      cluster_count: self.scene.get_cluster_count(),
      row_count: self.scene.get_row_count(),
      batch_count: StaticBatch::COUNT as u32,
      impostor_count: self.scene.get_impostor_count(),
      glod_start: threshold(options.lod.ssa_glod_start),
      glod_end: threshold(options.lod.ssa_glod_end),
      discard_below: options.lod.ssa_discard.powi(2) / screen,
      candidate_capacity: self.scene.get_list_capacity(),
      is_occluding: options.is_occlusion_culled as u32,
      lod_a: threshold(options.lod.ssa_a),
      lod_b: threshold(options.lod.ssa_b),
      is_impostors: options.lod.is_impostors as u32,
      hidden_groups: to_hidden_groups(options),
      pad: [0; 3],
      lod_origin: view.position.extend(1.0),
    };
    self.frame_view = (view.view, view.projection);
    self.grass.prepare(
      device,
      queue,
      passes.grass,
      &options.grass,
      view,
      self.params.discard_below,
      (self.started.elapsed().as_secs_f32(), options.is_windy),
    );
    // The campfires switch whatever of them is drawn, so their lights and particles follow them alike.
    self.campfires.prepare(options.is_campfire_lit);
    self.lights.prepare(
      queue,
      LightsFrame {
        camera: view,
        settings: &options.lights,
        lod: (self.params.glod_start, self.params.glod_end),
        contents: self.scene.get_contents(),
        sway: &to_sway(&self.scene, self.frame_sway),
        campfires: &mut self.campfires,
      },
    );
    self.frame_camera = *view;
    self.frame_sun = lighting.get_sun_direction();
    self.shadow_settings = options.shadows.clone();
    self.ambient_occlusion = options.ambient_occlusion;
    self.output = output;
    self.upscaling = options.upscaling;
    self.frame_debug_view = options.debug_view;
    self.is_wireframe = options.is_wireframe;
    self.frame_corrections = options.corrections;
    self.frame_occlusion = options.is_lit && options.ambient_occlusion.is_enabled;
    self.prepare_temporal(device, queue, passes, view);
    self.prepare_smoothing(device, passes, options.antialiasing);
    self.prepare_upscale(device, queue, passes);
    self.lights_settings = options.lights;

    queue.write_buffer(
      &self.occlusion_uniform,
      0,
      bytemuck::bytes_of(&AmbientOcclusionUniform::new(
        &options.ambient_occlusion,
        view.projection,
        (width.div_ceil(2), height.div_ceil(2)),
      )),
    );
    queue.write_buffer(&self.cull_params, 0, bytemuck::bytes_of(&self.params));
    self.prepare_sorted(device, queue, passes, view, self.scene.get_generation());

    if let Some((pyramid, _)) = &self.pyramid {
      let (history_view, history_projection): (Mat4, Mat4) = self.history.unwrap_or(self.frame_view);

      queue.write_buffer(
        &self.occlusion,
        0,
        bytemuck::bytes_of(&StaticOcclusionUniform {
          view: history_view,
          projection: history_projection,
          size: Vec2::new(pyramid.width as f32, pyramid.height as f32),
          levels: pyramid.levels,
          has_history: self.history.is_some() as u32,
        }),
      );
    }
    queue.write_buffer(
      &self.lighting,
      0,
      bytemuck::bytes_of(&LightingUniform::new(lighting, view.view, options, &frame)),
    );

    self.particles.step(view, options, &mut self.campfires);

    if let Some(targets) = &self.targets {
      self.particles.upload(
        device,
        queue,
        passes.particles,
        &self.lighting,
        (targets, self.targets_epoch),
      );
    }

    self.write_present(queue, options);
  }

  /// Writes what the present pass reads, once the frame knows what draws into the distortion target: `def_distort`
  /// while the water or a particle does, nothing otherwise.
  fn write_present(&self, queue: &wgpu::Queue, options: &RenderViewOptions) {
    let Some(targets) = &self.targets else {
      return;
    };
    let water: &RenderWaterSettings = &options.water;
    let is_water_distorting: bool = water.is_enabled && water.is_distorted && options.is_lit;
    let is_distorting: bool = !options.is_wireframe && (is_water_distorting || self.particles.is_distorting());

    queue.write_buffer(
      &self.present,
      0,
      bytemuck::bytes_of(&PresentUniform::new(
        self.frame_debug_view,
        self.frame_occlusion,
        !targets.is_sized(self.output.width, self.output.height),
        if is_distorting { water.distortion } else { 0.0 },
        self.output,
        &self.frame_corrections,
        self.selection_color,
      )),
    );
  }

  /// Writes this frame's rain, while the weather rains and the view shows it: the splash's model built for the
  /// level's weather, the uniform at the cover last drawn, and what the draw binds.
  fn prepare_rain(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    passes: LevelPasses<'_>,
    (lighting, weather): (&RenderLighting, Option<&Arc<RenderLevelWeather>>),
    options: &RenderViewOptions,
    weather_textures: &WeatherTextureCache,
  ) {
    self.rain_draw = None;

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

    if self.splash.as_ref().is_none_or(|(built, _)| *built != key) {
      self.splash = Some((key, WeatherModelBuffers::new(device, rain.drop.as_ref())));
    }

    let Some((_, splash)) = &self.splash else {
      return;
    };
    let uniform: RainUniform = RainUniform::new(
      &rainfall,
      self.rain_cover.get_window(),
      self.started.elapsed().as_secs_f32(),
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

    self.rain_draw = Some((uniform.count, splash.index_count));

    let Some(wet) = weather.and_then(|weather| weather.wet.as_ref()) else {
      return;
    };
    let Some(targets) = &self.targets else {
      return;
    };
    let wet_key: (u64, u64) = (self.targets_epoch, weather_textures.get_generation());

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
    self.thunder_draw = None;

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

    if self.thunder_models.as_ref().is_none_or(|(built, _)| *built != key) {
      let models: Vec<WeatherModelBuffers> = thunder
        .models
        .iter()
        .map(|model| WeatherModelBuffers::new(device, Some(&model.mesh)))
        .collect();

      self.thunder_models = Some((key, models));
    }

    let Some((_, models)) = &self.thunder_models else {
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
      let buffers: &WeatherModelBuffers = model.map_or(&self.no_model, |(buffers, _)| buffers);
      let model_texture: &str = model.map_or("", |(_, model)| model.mesh.texture.as_str());
      let groups: [wgpu::BindGroup; 3] = [
        passes
          .thunder
          .create_bind_group(device, &self.thunder, flat(model_texture), buffers),
        passes
          .thunder
          .create_bind_group(device, &self.thunder, flat(&bolt.top.texture), &self.no_model),
        passes
          .thunder
          .create_bind_group(device, &self.thunder, flat(&bolt.center.texture), &self.no_model),
      ];

      self.thunder_groups = Some((group_key, groups));
    }

    self.thunder_draw = Some((
      [
        model.map_or(XraySurfaceDraw::Opaque, |(_, model)| model.draw),
        bolt.top.draw,
        bolt.center.draw,
      ],
      model.map_or(0, |(buffers, _)| buffers.index_count),
    ));
  }

  /// Culls the scene and draws it into the G-buffer: what last frame's depth does not hide, then, culling occlusion,
  /// what this frame's first draw does not hide of the rest, leaving this frame's depth reduced for the next.
  #[allow(clippy::too_many_arguments)]
  pub fn record(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    passes: LevelPasses<'_>,
    view_layout: &wgpu::BindGroupLayout,
    view: &ViewBinding,
    textures: &TextureCache,
  ) {
    let (Some(targets), Some((pyramid, pyramid_groups)), Some((_, cull_group)), Some((_, draw_groups))) =
      (&self.targets, &self.pyramid, &self.cull_group, &self.draw_groups)
    else {
      return;
    };
    let texture_group: &wgpu::BindGroup = textures.get_bind_group();

    targets.clear_distortion(encoder);

    // The draw arguments a forward pass replays: the early phase's, and the late phase's where occlusion culls.
    let list_args: Vec<&wgpu::Buffer> = if self.params.is_occluding != 0 {
      vec![&self.scene.args, &self.scene.late]
    } else {
      vec![&self.scene.args]
    };
    let timer: &mut PassTimer = &mut self.timer;

    timer.begin(encoder);
    self.grass.plant(encoder, passes.grass);
    timer.mark(encoder, "grass planting");
    passes.cull.dispatch_early(encoder, view, cull_group, &self.params);
    timer.mark(encoder, "cull");
    passes.gbuffer.draw(
      encoder,
      targets,
      view,
      draw_groups,
      texture_group,
      &self.scene.args,
      true,
    );
    timer.mark(encoder, "g-buffer");

    if self.params.is_occluding != 0 {
      passes.pyramid.dispatch(encoder, pyramid, pyramid_groups);
      passes.cull.dispatch_late(encoder, view, cull_group, &self.scene);
      passes.gbuffer.draw(
        encoder,
        targets,
        view,
        draw_groups,
        texture_group,
        &self.scene.late,
        false,
      );
      self.history = Some(self.frame_view);
      timer.mark(encoder, "occlusion");
    }

    self.stats.record(encoder, &self.scene.args, StaticScene::STATS_OFFSET);
    if !self.is_wireframe {
      self.grass.draw(encoder, passes.grass, (targets, view), texture_group);
      timer.mark(encoder, "grass");
    }

    if self.is_wallmarked && !self.is_wireframe {
      passes
        .composited
        .draw_wallmarks(encoder, targets, view, draw_groups, texture_group, &list_args);
      timer.mark(encoder, "wall marks");
    }

    if let Some((pyramid, _)) = &self.pyramid {
      let frame: ShadowFrame<'_> = ShadowFrame {
        scene: &self.scene,
        camera: &self.frame_camera,
        settings: &self.shadow_settings,
        sun_direction: self.frame_sun,
        sway: to_sway(&self.scene, self.frame_sway),
        cull_params: &self.cull_params,
        params: &self.params,
        pyramid: &pyramid.view,
        occlusion: &self.occlusion,
        targets_epoch: self.targets_epoch,
        textures,
      };

      self.shadows.record(device, queue, encoder, passes, view_layout, &frame);
      timer.mark(encoder, "sun shadows");

      if self.rain_draw.is_some() {
        self.rain_cover.record(device, queue, encoder, passes, &frame);
        timer.mark(encoder, "rain cover");
      }

      self.lights.record_shadows(device, queue, encoder, passes, &frame);
      timer.mark(encoder, "light shadows");
    }

    // The rain wets the G-buffer before any light is drawn over it.
    if self.rain_draw.is_some()
      && let Some((_, wet_groups)) = &self.wet_groups
    {
      passes.wet.draw(encoder, targets, view, wet_groups);
      timer.mark(encoder, "wet");
    }

    if let Some((_, groups)) = &self.light_groups {
      passes.sun.draw(encoder, targets, view, &groups.sun);
      timer.mark(encoder, "sun");

      self.lights.clear_overflow(encoder);

      if self.lights.get_count() > 0 {
        passes
          .lights
          .draw(encoder, targets, view, &groups.lights, textures.get_bind_group());
        timer.mark(encoder, "lights");
      }

      self.lights.record_overflow(encoder);

      if self.ambient_occlusion.is_enabled {
        passes.ambient_occlusion.draw(
          encoder,
          targets,
          view,
          &groups.occlusion,
          self.ambient_occlusion.quality,
        );
        timer.mark(encoder, "ambient occlusion");
      }

      if let Some((_, sky_group)) = &self.sky_group {
        if self.is_hazing {
          passes.sky_haze.draw(encoder, targets, &groups.haze, sky_group);
          timer.mark(encoder, "haze");
        }

        passes.combine.draw(encoder, targets, view, &groups.combine, sky_group);
        timer.mark(encoder, "combine");
      }

      // FSR 2's reactive mask is what the water and the blended surfaces change of the frame drawn so far.
      if let Some((fsr, _)) = &self.fsr {
        encoder.copy_texture_to_texture(
          targets.scene_texture.as_image_copy(),
          fsr.opaque_texture.as_image_copy(),
          fsr.opaque_texture.size(),
        );
      }

      if self.water_settings.is_enabled
        && !self.is_wireframe
        && let Some((_, water_group)) = &self.water_group
      {
        passes.water.draw(
          encoder,
          targets,
          view,
          draw_groups,
          texture_group,
          water_group,
          &list_args,
        );
        timer.mark(encoder, "water");
      }

      if !self.is_wireframe
        && let Some((_, sky_group)) = &self.sky_group
      {
        passes.composited.draw(
          encoder,
          targets,
          view,
          draw_groups,
          texture_group,
          (&groups.composited, sky_group),
          &list_args,
          (self.sorted_group.as_ref().map(|(_, group)| group), self.sorted_count),
        );
        timer.mark(encoder, "composited");
      }

      if !self.is_wireframe
        && self
          .particles
          .record(encoder, passes.particles, targets, view, texture_group)
      {
        timer.mark(encoder, "particles");
      }

      if self.is_shafted {
        passes.sun_shafts.draw(encoder, targets, view, &groups.sun_shafts);
        timer.mark(encoder, "sun shafts");
      }

      if let (Some(counts), Some((_, rain_group))) = (self.rain_draw, &self.rain_group) {
        passes.rain.draw(encoder, targets, view, rain_group, counts);
        timer.mark(encoder, "rain");
      }

      if let (Some(draws), Some((_, thunder_groups))) = (self.thunder_draw, &self.thunder_groups) {
        passes.thunder.draw(encoder, targets, view, thunder_groups, draws);
        timer.mark(encoder, "thunder");
      }

      self.flares.record(encoder, passes.flares, targets, view);
      timer.mark(encoder, "flares");

      if let Some(smoothing) = &self.smoothing {
        let target: &SmoothingTarget = &smoothing.target;

        match &smoothing.smaa {
          Some(smaa) => passes.smaa.draw(encoder, smaa, &smoothing.groups, &target.view),
          None => passes.fxaa.draw(encoder, &smoothing.groups[0], &target.view),
        }

        encoder.copy_texture_to_texture(
          target.texture.as_image_copy(),
          targets.scene_texture.as_image_copy(),
          target.texture.size(),
        );
        timer.mark(encoder, "smoothing");
      }

      // The resolved frame goes where the present pass reads it: the upscaled frame, or the scene drawn at its size.
      if let Some((fsr, groups)) = &mut self.fsr {
        let resolved: &wgpu::Texture = self
          .upscale
          .as_ref()
          .map_or(&targets.scene_texture, |(upscale, _)| &upscale.textures[0]);
        let history: &wgpu::Texture = &fsr.history_textures[fsr.index];

        passes
          .fsr
          .draw(encoder, fsr, groups, &mut |encoder, name| timer.mark(encoder, name));
        encoder.copy_texture_to_texture(history.as_image_copy(), resolved.as_image_copy(), history.size());
        fsr.swap();
        timer.mark(encoder, "fsr2 output");
      } else if let Some((history, groups)) = &mut self.temporal {
        let index: usize = history.index;
        let resolved: &wgpu::Texture = self
          .upscale
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
        timer.mark(encoder, "temporal");
      } else if let Some((upscale, groups)) = &self.upscale {
        passes.upscale.draw_easu(encoder, upscale, &groups[0]);
        timer.mark(encoder, "upscale");
      }

      if let Some((upscale, groups)) = &self.upscale
        && self.upscaling.is_sharpened()
      {
        passes.upscale.draw_rcas(encoder, upscale, &groups[1]);
        timer.mark(encoder, "sharpen");
      }

      if self.exposure.is_adapting() {
        passes.exposure.dispatch(encoder, &groups.exposure);
        timer.mark(encoder, "exposure");
      }
    }

    timer.finish(encoder);
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
    let target: &PickTarget = self.pick_target.get_or_insert_with(|| PickTarget::new(device));
    let view: &ViewBinding = self
      .pick_view
      .get_or_insert_with(|| ViewBinding::new(device, view_layout));

    view.write(queue, camera);
    draw.pick(
      encoder,
      target,
      view,
      draw_groups,
      textures.get_bind_group(),
      &[&self.scene.args, &self.scene.late],
    );
  }

  /// Reads a recorded pick back, once its frame was submitted, and names what it met.
  pub fn resolve_pick(
    &self,
    device: &wgpu::Device,
    unproject: impl Fn(f32) -> Vec3,
  ) -> XrfResult<Option<RenderLevelHit>> {
    let Some(target) = &self.pick_target else {
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

    let Some((info, instance)) = self.scene.resolve_pick(cluster, place) else {
      return Ok(None);
    };

    if info.sector == StaticSlotInfo::NO_SECTOR {
      return Ok(
        self
          .scene
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
    self.is_temporal = options.antialiasing.is_temporal() && options.debug_view == RenderDebugView::Final;
    self.is_fsr = self.is_temporal && options.antialiasing == RenderAntialiasing::Fsr2;
    self.frame_phases = TemporalJitter::get_phases(ratio);
    self.frame_jitter = if self.is_temporal {
      self.jitter.next(ratio)
    } else {
      Vec2::ZERO
    };

    self.frame_jitter
  }

  /// This frame's unjittered view projection and the last one's, which the surfaces' motion is measured between; the
  /// same twice for a first frame.
  pub fn next_motion(&mut self, current: Mat4) -> (Mat4, Mat4) {
    (current, self.motion_previous.replace(current).unwrap_or(current))
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
    let Some(targets) = self.targets.as_ref().filter(|_| self.is_temporal && !self.is_fsr) else {
      self.temporal = None;
      self.temporal_previous = None;
      self.prepare_fsr(device, queue, passes, view);

      return;
    };

    self.fsr = None;
    let (width, height): (u32, u32) = (self.output.width.max(1), self.output.height.max(1));

    if self
      .temporal
      .as_ref()
      .is_some_and(|(history, _)| !history.is_sized(width, height))
    {
      self.temporal = None;
    }

    let (history, _) = self.temporal.get_or_insert_with(|| {
      let history: TemporalHistory = TemporalHistory::new(device, width, height);
      let groups: [wgpu::BindGroup; 2] =
        passes
          .temporal
          .create_bind_groups(device, targets, &history, &self.temporal_uniform);

      (history, groups)
    });
    let current: Mat4 = view.get_view_projection();
    let (previous, previous_view): (Mat4, Mat4) = self.temporal_previous.unwrap_or((current, view.view));

    queue.write_buffer(
      &self.temporal_uniform,
      0,
      bytemuck::bytes_of(&TemporalUniform::new(
        current,
        previous,
        previous_view,
        self.frame_jitter,
        history.is_valid && self.temporal_previous.is_some(),
      )),
    );
    self.temporal_previous = Some((current, view.view));
  }

  /// Makes FSR 2's targets while it resolves, dropping them otherwise, and writes its constants.
  fn prepare_fsr(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, passes: LevelPasses<'_>, view: &CameraView) {
    let Some(targets) = self.targets.as_ref().filter(|_| self.is_fsr) else {
      self.fsr = None;

      return;
    };
    let render: (u32, u32) = (targets.width, targets.height);
    let display: (u32, u32) = (self.output.width.max(1), self.output.height.max(1));

    if self.fsr.as_ref().is_some_and(|(fsr, _)| !fsr.is_sized(render, display)) {
      self.fsr = None;
    }

    let (fsr, _) = self.fsr.get_or_insert_with(|| {
      let fsr: FsrTargets = FsrTargets::new(device, render, display, ViewTargets::SCENE);
      let groups: FsrGroups = passes.fsr.create_bind_groups(device, targets, &fsr, &self.fsr_uniform);

      (fsr, groups)
    });

    queue.write_buffer(
      &self.fsr_uniform,
      0,
      bytemuck::bytes_of(&FsrUniform::new(
        (render, display),
        self.frame_jitter,
        view,
        (FsrTargets::get_luma_mip_size(render), self.frame_phases),
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
    let hidden: u32 = self.params.hidden_groups;
    let mut places: Vec<(f32, &StaticSortedPlace)> = self
      .scene
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

    self.sorted_count = entries.len() as u32;

    if bytes.is_empty() {
      return;
    }

    if self
      .sorted_list
      .as_ref()
      .is_none_or(|buffer| buffer.size() < bytes.len() as u64)
    {
      self.sorted_list = Some(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("sorted composited"),
        size: (bytes.len() as u64).next_power_of_two(),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }));
      self.sorted_epoch += 1;
    }

    let Some(buffer) = &self.sorted_list else {
      return;
    };

    queue.write_buffer(buffer, 0, bytes);

    let key: (u64, u64) = (generation, self.sorted_epoch);

    if self.sorted_group.as_ref().is_none_or(|(it, _)| *it != key) {
      let [_, _, model] = passes.gbuffer.create_layout_groups(device, &self.scene, buffer);

      self.sorted_group = Some((key, model));
    }
  }

  /// Makes the smoothing pass's targets while one smooths the frame as drawn, dropping them otherwise.
  fn prepare_smoothing(&mut self, device: &wgpu::Device, passes: LevelPasses<'_>, mode: RenderAntialiasing) {
    let is_smoothed: bool = matches!(mode, RenderAntialiasing::Fxaa | RenderAntialiasing::Smaa);
    let Some(targets) = self
      .targets
      .as_ref()
      .filter(|_| is_smoothed && self.frame_debug_view == RenderDebugView::Final)
    else {
      self.smoothing = None;

      return;
    };

    if self
      .smoothing
      .as_ref()
      .is_none_or(|it| it.epoch != self.targets_epoch || it.mode != mode)
    {
      let target: SmoothingTarget = SmoothingTarget::new(device, targets.width, targets.height, ViewTargets::SCENE);
      let (smaa, groups): (Option<SmaaTargets>, Vec<wgpu::BindGroup>) = if mode == RenderAntialiasing::Smaa {
        let smaa: SmaaTargets = SmaaTargets::new(device, targets.width, targets.height);
        let groups: [wgpu::BindGroup; 3] = passes.smaa.create_bind_groups(device, targets, &smaa);

        (Some(smaa), groups.into())
      } else {
        (None, vec![passes.fxaa.create_bind_group(device, targets)])
      };

      self.smoothing = Some(LevelSmoothing {
        epoch: self.targets_epoch,
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
    let Some(targets) = &self.targets else {
      return;
    };
    let output: RenderRect = self.output;
    let is_upscaled: bool = !targets.is_sized(output.width, output.height);

    if !is_upscaled {
      self.upscale = None;
    } else if self
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

      self.upscale = Some((upscale, groups));
      self.upscale_epoch += 1;
    }

    // An upscale made before the targets were reads a scene since dropped.
    if let Some((upscale, groups)) = &mut self.upscale
      && self
        .present_group
        .as_ref()
        .is_none_or(|(key, _)| key.0 != self.targets_epoch)
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
        sharpness: self.upscaling.get_sharpness(),
        pad: 0.0,
      }),
    );
    if self
      .overlay_group
      .as_ref()
      .is_none_or(|(epoch, _)| *epoch != self.targets_epoch)
    {
      let group: wgpu::BindGroup = passes
        .overlay
        .create_bind_group(device, targets, &self.present, &self.lighting);

      self.overlay_group = Some((self.targets_epoch, group));
    }

    let shown: usize = usize::from(self.upscaling.is_sharpened());
    let key: (u64, u64, usize) = (
      self.targets_epoch,
      if is_upscaled { self.upscale_epoch } else { 0 },
      shown,
    );

    if self.present_group.as_ref().is_none_or(|(it, _)| *it != key) {
      let upscaled: Option<&wgpu::TextureView> = self.upscale.as_ref().map(|(upscale, _)| &upscale.views[shown]);
      let group: wgpu::BindGroup = passes
        .present
        .create_bind_group(device, targets, &self.present, upscaled);

      self.present_group = Some((key, group));
    }
  }

  /// Asks for the counts and pass timestamps recorded with the frame just submitted.
  pub fn request_stats(&self) {
    self.stats.request();
    self.timer.request();
    self.lights.request_report();
  }

  /// Times each pass of its frames on the GPU, where the device can.
  pub fn set_timed(&mut self, is_timed: bool) {
    self.timer.set_enabled(is_timed);
  }

  /// Whether its passes are timed, and each one's mean GPU milliseconds since this was last asked.
  pub fn take_timings(&mut self) -> (bool, Vec<RenderPassCost>) {
    (self.timer.is_timing(), self.timer.take())
  }

  /// Adds the passes timed since the last frame to the span the next report averages.
  pub fn collect_timings(&mut self) {
    self.timer.collect();
  }

  /// Stands every skinned object as asked from the next frame on.
  pub fn set_model_pose(&mut self, pose: &RenderModelPose) {
    if self.model_pose != *pose {
      self.model_pose = pose.clone();
    }
  }

  /// Writes every skinned object's bone matrices for this frame, and the last frame's beside them; a motion still on
  /// its way poses the bind pose meanwhile.
  fn pose_skeletons(&mut self, queue: &wgpu::Queue) {
    if self.skeletons.is_empty() {
      return;
    }

    let pose: &RenderModelPose = &self.model_pose;
    let motion: Option<&RenderMotion> = match &pose.motion {
      Some(name) => self.motions.get(&self.source, name),
      None => None,
    };

    for (object, skeleton) in &mut self.skeletons {
      let (current, previous) = skeleton.pose(motion, pose.frame, &pose.hidden_bones);

      self.scene.write_pose(queue, *object, &current, &previous);
    }
  }

  /// Every skinned object's bones as segments in renderer space, child then parent, where this frame poses them.
  pub fn list_skeleton_segments(&self) -> Vec<(Vec3, Vec3)> {
    self
      .skeletons
      .iter()
      .flat_map(|(object, skeleton)| {
        let place: Mat4 = self.scene.get_object_transform(*object).unwrap_or(Mat4::IDENTITY);

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
      skipped: self.skipped.clone(),
      sectors: self.failed_sectors.clone(),
      models: self.spawn.list_failures(),
    }
  }

  /// A spawned object's bounding sphere in renderer space, once its model is in the scene.
  pub fn get_object_sphere(&self, object: u32) -> Option<Vec4> {
    self.scene.get_object_sphere(object)
  }

  /// What the level's particle systems came to since the last report.
  pub fn take_particles_report(&mut self) -> RenderParticlesReport {
    self.particles.take_report()
  }

  /// Milliseconds the last sector taken in took to put into the scene.
  pub fn get_sector_time(&self) -> f32 {
    self.sector_time
  }

  /// What its frames are drawn with, as resolved from what `options` asked; the weather's light is the viewport's to add.
  pub fn describe_applied(&self, options: &RenderViewOptions) -> RenderAppliedReport {
    let antialiasing: RenderAntialiasing = if self.is_temporal {
      options.antialiasing
    } else {
      self.smoothing.as_ref().map_or(RenderAntialiasing::None, |it| it.mode)
    };
    let cascades: usize = self.shadow_settings.get_cascade_count();

    RenderAppliedReport {
      antialiasing,
      render_scale: self.upscaling.scale,
      shadows: (cascades > 0).then(|| RenderAppliedShadows {
        cascades: self.shadow_settings.cascades[..cascades].to_vec(),
        resolution: self.shadows.get_maps().resolution,
        filter: self.shadow_settings.filter,
      }),
      ambient_occlusion: self.frame_occlusion.then_some(self.ambient_occlusion.quality),
      lights: self.lights_settings.is_enabled.then_some(self.lights_settings),
      grass: self.grass.get_applied(&options.grass),
      is_water: options.water.is_enabled,
      environment: None,
      sun: self
        .frame_sun_sprite
        .as_ref()
        .and_then(|_| self.flares.get_shown())
        .map(str::to_owned),
    }
  }

  /// The static draws' pools and what the latest counted frame's cull kept and hid, and its lights.
  pub fn take_stats(&mut self) -> (RenderStaticReport, RenderLightsReport) {
    let [kept_clusters, kept_triangles, occluded_clusters, occluded_triangles] = self.stats.take();
    let pools: RenderStaticReport = self.scene.get_pools();
    let phases: u32 = if self.params.is_occluding != 0 { 2 } else { 1 };

    (
      RenderStaticReport {
        surface_list: RenderPoolUse {
          used: kept_clusters,
          ..pools.surface_list
        },
        commands: StaticBatch::list_deferred().count() as u32 * phases + u32::from(self.params.is_impostors != 0),
        kept_clusters,
        kept_triangles,
        occluded_clusters,
        occluded_triangles,
        ..pools
      },
      self.lights.take_report(),
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
        .get_object_box(object)
        .map(|(transform, corners)| to_box_lines(transform, corners, selection.color)),
      RenderSelectionTarget::Surface { .. } => None,
    });

    if self.overlays.as_ref().is_none_or(|it| it.version != version) || self.overlays_box != boxed {
      let all: Vec<RenderOverlay> = overlays.iter().cloned().chain(boxed.clone()).collect();

      self.overlays = Some(LevelOverlays::new(device, &all, version));
      self.overlays_box = boxed;
    }
  }

  /// What a selection marks in the scene now, resolved again only once the target or the scene changed; none where
  /// nothing is selected or what it names is not in the scene yet.
  pub fn resolve_selection(&mut self, selection: Option<&RenderSelection>) -> Option<&StaticSelection> {
    self.selection_color = None;

    let selection: &RenderSelection = selection?;
    let generation: u64 = self.scene.get_generation();

    if self
      .selection
      .as_ref()
      .is_none_or(|(target, at, _)| *target != selection.target || *at != generation)
    {
      self.selection = Some((
        selection.target,
        generation,
        self.scene.resolve_selection(&selection.target),
      ));
    }

    let resolved: &StaticSelection = self.selection.as_ref()?.2.as_ref()?;

    self.selection_color = Some(selection.color);

    Some(resolved)
  }

  /// What it draws over its frame, with the bind group drawing it, once both are made.
  pub fn get_overlays(&self) -> Option<(&wgpu::BindGroup, &LevelOverlays)> {
    Some((&self.overlay_group.as_ref()?.1, self.overlays.as_ref()?))
  }

  /// The size its scene is rendered at, once its targets are made.
  pub fn get_render_size(&self) -> Option<(u32, u32)> {
    self.targets.as_ref().map(|it| (it.width, it.height))
  }

  /// What puts the level's finished scene into the window, once its targets are made.
  pub fn get_present_group(&self) -> Option<&wgpu::BindGroup> {
    self.present_group.as_ref().map(|(_, group)| group)
  }

  /// Every environment slot it samples, so the cubes no scene samples can be freed.
  pub fn list_environment_slots(&self) -> impl Iterator<Item = u32> + '_ {
    self.scene.environment_slots.iter().copied()
  }

  /// Every texture slot it samples, so the slots no scene samples can be freed.
  pub fn list_texture_slots(&self) -> impl Iterator<Item = u32> + '_ {
    self
      .scene
      .texture_slots
      .iter()
      .copied()
      .chain(self.lights.get_projectors().iter().copied())
      .chain(self.particles.get_texture_slots().iter().copied())
  }

  /// Bytes its scene's growing buffers hold on the GPU.
  pub fn get_buffer_bytes(&self) -> u64 {
    self.scene.get_buffer_bytes()
  }

  /// Whether it draws this source.
  pub fn is_showing(&self, source: &Arc<dyn RenderLevelSource>) -> bool {
    Arc::ptr_eq(&self.source, source)
  }

  /// Whether everything it opens with is resident, so it draws as it will.
  pub fn is_ready(&self, textures: &TextureCache) -> bool {
    self.describe_load(textures).is_ready
  }

  /// How far the level has loaded, when that changed since it was last asked.
  pub fn take_report(&mut self, textures: &TextureCache) -> Option<RenderLoadReport> {
    let report: RenderLoadReport = self.describe_load(textures);

    if self.reported == Some(report) {
      return None;
    }

    self.reported = Some(report);

    Some(report)
  }

  /// How far the level has loaded: its sectors taken in or failed, its spawn, its grass, lights and particles read, and
  /// every texture it samples settled; and how long each took.
  pub fn describe_load(&self, textures: &TextureCache) -> RenderLoadReport {
    let settled: u32 = textures.count_settled(self.list_texture_slots());
    let total: u32 = self.list_texture_slots().count() as u32;
    let is_read: bool =
      self.spawn.is_done() && self.grass.is_loaded() && self.lights.is_loaded() && self.particles.is_loaded();

    RenderLoadReport {
      sectors: self.scene.sectors.len() as u32,
      sectors_total: self.loader.get_total(),
      bytes: self.scene.get_bytes(),
      textures: settled,
      textures_total: total,
      is_ready: self.are_sectors_done() && is_read && settled == total,
      durations: self.load_durations,
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
