use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, Vec2, Vec3};
use xrf_engine_target::XrayEngine;
use xrf_error::XrfResult;
use xrf_material::XraySurfaceDraw;

use xrf_math::EPS_S;

use crate::camera::camera_view::CameraView;
use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_level_hit::RenderLevelHit;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::contract::render_spawn_category::RenderSpawnCategory;
use crate::contract::render_surface_geometry::RenderSurfaceGeometry;
use crate::contract::render_texture_report::RenderTextureReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_water_settings::RenderWaterSettings;
use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::pick_target::PickTarget;
use crate::frame::stats_readback::StatsReadback;
use crate::frame::view_exposure::ViewExposure;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::host::render_rain::RenderRain;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::ambient_occlusion_uniform::AmbientOcclusionUniform;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::grass_pass::GrassPass;
use crate::pass::level_passes::LevelPasses;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::rain_bindings::RainBindings;
use crate::pass::rain_uniform::RainUniform;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_draw_groups::StaticDrawGroups;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;
use crate::pass::thunder_uniform::ThunderUniform;
use crate::pass::view_binding::ViewBinding;
use crate::pass::view_light_groups::ViewLightGroups;
use crate::pass::water_uniform::WaterUniform;
use crate::pass::wet_uniform::WetUniform;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::level_grass::LevelGrass;
use crate::scene::level::level_lights::LevelLights;
use crate::scene::level::level_loader::LevelLoader;
use crate::scene::level::level_shadows::LevelShadows;
use crate::scene::level::rain_cover::RainCover;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::shadow_sway::ShadowSway;
use crate::scene::level::spawn_loader::SpawnLoader;
use crate::scene::level::surface_tally::SurfaceTally;
use crate::scene::level::weather_model_buffers::WeatherModelBuffers;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::static_scene::static_slot_info::StaticSlotInfo;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// What a pick's texel says it met: a cluster, by its index and place, or an impostor, by its index.
const PICKED_CLUSTER: u32 = 1;
const PICKED_IMPOSTOR: u32 = 2;

/// Sectors put on the GPU at most each frame, so a level's open spreads over frames rather than stalling one.
const SECTORS_PER_FRAME: usize = 4;

/// Spawned models put into the scene at most in one frame.
const MODELS_PER_FRAME: usize = 16;

/// The engine's progressive mesh thresholds, in screen area before the screen is applied: whole above the first,
/// coarsest below the second.
const GLOD_START: f32 = 256.0;
const GLOD_END: f32 = 64.0;

/// `r_ssaLOD_A` and `r_ssaLOD_B`: a clump's impostor draws below the first, its trees above the second.
const SSA_LOD_A: f32 = 64.0;
const SSA_LOD_B: f32 = 48.0;

/// `r_ssaDISCARD`: an instanced place smaller on screen than this is not drawn.
const SSA_DISCARD: f32 = 3.5;

/// What a sky's bind group binds: the weather textures' generation, and the references of its six slots.
type SkyGroupKey = (u64, [Option<String>; 6]);

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
  /// The sky's textures as bound, with the cache's generation and the references they bind.
  sky_group: Option<(SkyGroupKey, wgpu::BindGroup)>,
  /// Whether this frame blurs the sky into the haze map the distance fades into.
  is_hazing: bool,
  /// Whether this frame lays the wall marks into the albedo.
  is_wallmarked: bool,
  /// Bumped whenever the sky's bind group is made again, which the water's follows.
  sky_version: u64,
  water: wgpu::Buffer,
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
  /// When the level was first shown, which the clouds drift from.
  started: Instant,
  shadows: LevelShadows,
  lights: LevelLights,
  lights_settings: RenderLightsSettings,
  occlusion_uniform: wgpu::Buffer,
  ambient_occlusion: RenderAmbientOcclusionSettings,
  exposure: ViewExposure,
  params: StaticCullParams,
  stats: StatsReadback,
  pick_target: Option<PickTarget>,
  pick_view: Option<ViewBinding>,
  surfaces: SurfaceTally,
  failed: u32,
  reported: Option<RenderLoadReport>,
}

impl LevelView {
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    view_layout: &wgpu::BindGroupLayout,
    source: Arc<dyn RenderLevelSource>,
  ) -> Self {
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
      loader: LevelLoader::start(Arc::clone(&source)),
      spawn: SpawnLoader::start(Arc::clone(&source)),
      grass: LevelGrass::new(device, &source),
      lights: LevelLights::new(device, view_layout, scene.args.size(), &source),
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
      sky_group: None,
      is_hazing: false,
      is_wallmarked: true,
      sky_version: 0,
      water: uniform("water", size_of::<WaterUniform>()),
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
      started: Instant::now(),
      shadows: LevelShadows::new(device),
      lights_settings: RenderLightsSettings::default(),
      occlusion_uniform: uniform("ambient occlusion", size_of::<AmbientOcclusionUniform>()),
      ambient_occlusion: RenderAmbientOcclusionSettings::default(),
      exposure: ViewExposure::new(device, queue),
      params: StaticCullParams::default(),
      stats: StatsReadback::new(device),
      pick_target: None,
      pick_view: None,
      surfaces: SurfaceTally::default(),
      failed: 0,
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
      weather_textures.request_sky(&lighting.sky, options.is_clouded, &assets);
    } else {
      // The irradiance cubes light the hemisphere whether or not the sky is drawn.
      for reference in lighting.sky.environments.iter().flatten() {
        weather_textures.request(reference, WeatherTextureKind::Cube, &assets);
      }
    }

    self.lights.poll(textures, &assets);

    if let Some(slots) = self.grass.poll(device, grass_pass, textures, &assets) {
      self.scene.texture_slots.extend(slots);
    }

    for (_, package) in self.loader.take(SECTORS_PER_FRAME) {
      match package {
        Ok((package, tally)) => {
          self.scene.add_sector(
            device,
            queue,
            encoder,
            textures,
            &assets,
            self.source.get_surfaces(),
            &package,
          );
          self.surfaces.merge(tally);
        }
        Err(_) => self.failed += 1,
      }
    }

    for (model, places) in self.spawn.take(MODELS_PER_FRAME) {
      self
        .scene
        .add_model(device, queue, encoder, textures, &assets, &model, &places);
    }
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
    (width, height): (u32, u32),
    field_of_view: f32,
    options: &RenderViewOptions,
    (lighting, weather): (&RenderLighting, Option<&Arc<RenderLevelWeather>>),
    weather_textures: &WeatherTextureCache,
  ) {
    if !self.targets.as_ref().is_some_and(|it| it.is_sized(width, height)) {
      let targets: ViewTargets = ViewTargets::new(device, width, height);
      let pyramid: DepthPyramid = DepthPyramid::new(device, width, height);
      let groups: Vec<wgpu::BindGroup> = passes.pyramid.create_bind_groups(device, &targets.depth, &pyramid);

      self.targets = Some(targets);
      self.pyramid = Some((pyramid, groups));
      self.targets_epoch += 1;
      // A pyramid of another size holds no depth this frame can be tested against.
      self.history = None;
      self.light_groups = None;
    }

    self.scene.reset_draws(device, queue, encoder);

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
        exposure: passes.exposure.create_bind_group(device, targets, &self.exposure),
        present: passes.present.create_bind_group(device, targets),
      };

      self.light_groups = Some((shadow_epoch, groups));
    }

    self.exposure.prepare(queue, &options.exposure, Instant::now());

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
      ],
    );

    if self.sky_group.as_ref().is_none_or(|(key, _)| *key != sky_key) {
      self.sky_group = Some((sky_key, passes.sky.create_bind_group(device, weather_textures, sky)));
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
    let wind: WindUniform = WindUniform::new(lighting.trees.as_ref().filter(|_| options.is_windy), sway_time);

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
    };

    self.is_hazing = options.is_lit && options.is_sky_visible && options.is_sky_hazed;
    self.is_wallmarked = options.is_wallmarked;

    if !options.is_occlusion_culled {
      self.history = None;
    }

    // The engine's screen: the viewport's pixels, widened for a lens narrower than its 90 degrees.
    let screen: f32 =
      (width * height) as f32 * (90.0 / field_of_view.max(1.0)).powi(2) * (EPS_S + options.geometry_lod);
    let threshold = |area: f32| -> f32 { (area / 3.0).powi(2) / screen };

    self.params = StaticCullParams {
      cluster_count: self.scene.get_cluster_count(),
      row_count: self.scene.get_row_count(),
      batch_count: StaticBatch::COUNT as u32,
      impostor_count: self.scene.get_impostor_count(),
      glod_start: threshold(GLOD_START),
      glod_end: threshold(GLOD_END),
      discard_below: SSA_DISCARD.powi(2) / screen,
      candidate_capacity: self.scene.get_list_capacity(),
      is_occluding: options.is_occlusion_culled as u32,
      lod_a: threshold(SSA_LOD_A),
      lod_b: threshold(SSA_LOD_B),
      is_impostors: options.is_impostors as u32,
      hidden_groups: [
        RenderSpawnCategory::Props,
        RenderSpawnCategory::Items,
        RenderSpawnCategory::Weapons,
        RenderSpawnCategory::Lamps,
      ]
      .into_iter()
      .filter(|category| !options.is_spawned(*category))
      .fold(0, |hidden, category| hidden | (1 << (category.get_group() - 1))),
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
    self.lights.prepare(
      queue,
      view,
      &options.lights,
      (self.params.glod_start, self.params.glod_end),
      self.scene.get_contents(),
      &to_sway(&self.scene, self.frame_sway),
    );
    self.frame_camera = *view;
    self.frame_sun = lighting.get_sun_direction();
    self.shadow_settings = options.shadows.clone();
    self.ambient_occlusion = options.ambient_occlusion;
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

    self.grass.plant(encoder, passes.grass);
    passes.cull.dispatch_early(encoder, view, cull_group, &self.params);
    passes.gbuffer.draw(
      encoder,
      targets,
      view,
      draw_groups,
      texture_group,
      &self.scene.args,
      true,
    );

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
    }

    self.stats.record(encoder, &self.scene.args, StaticScene::STATS_OFFSET);
    self.grass.draw(encoder, passes.grass, (targets, view), texture_group);

    if self.is_wallmarked {
      passes
        .composited
        .draw_wallmarks(encoder, targets, view, draw_groups, texture_group, &self.list_args());
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

      if self.rain_draw.is_some() {
        self.rain_cover.record(device, queue, encoder, passes, &frame);
      }

      self.lights.record_shadows(device, queue, encoder, passes, &frame);
    }

    // The rain wets the G-buffer before any light is drawn over it.
    if self.rain_draw.is_some()
      && let Some((_, wet_groups)) = &self.wet_groups
    {
      passes.wet.draw(encoder, targets, view, wet_groups);
    }

    if let Some((_, groups)) = &self.light_groups {
      passes.sun.draw(encoder, targets, view, &groups.sun);

      if self.lights.get_count() > 0 {
        passes
          .lights
          .draw(encoder, targets, view, &groups.lights, textures.get_bind_group());
      }

      if self.ambient_occlusion.is_enabled {
        passes.ambient_occlusion.draw(
          encoder,
          targets,
          view,
          &groups.occlusion,
          self.ambient_occlusion.quality,
        );
      }

      if let Some((_, sky_group)) = &self.sky_group {
        if self.is_hazing {
          passes.sky_haze.draw(encoder, targets, &groups.haze, sky_group);
        }

        passes.combine.draw(encoder, targets, view, &groups.combine, sky_group);
      }

      if self.water_settings.is_enabled
        && let Some((_, water_group)) = &self.water_group
      {
        passes.water.draw(
          encoder,
          targets,
          view,
          draw_groups,
          texture_group,
          water_group,
          &self.list_args(),
        );
      }

      if let Some((_, sky_group)) = &self.sky_group {
        passes.composited.draw(
          encoder,
          targets,
          view,
          draw_groups,
          texture_group,
          (&groups.composited, sky_group),
          &self.list_args(),
        );
      }

      if let (Some(counts), Some((_, rain_group))) = (self.rain_draw, &self.rain_group) {
        passes.rain.draw(encoder, targets, view, rain_group, counts);
      }

      if let (Some(draws), Some((_, thunder_groups))) = (self.thunder_draw, &self.thunder_groups) {
        passes.thunder.draw(encoder, targets, view, thunder_groups, draws);
      }

      if self.exposure.is_adapting() {
        passes.exposure.dispatch(encoder, &groups.exposure);
      }
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

  /// The draw arguments a forward pass replays: the early phase's, and the late phase's where occlusion culls.
  fn list_args(&self) -> Vec<&wgpu::Buffer> {
    if self.params.is_occluding != 0 {
      vec![&self.scene.args, &self.scene.late]
    } else {
      vec![&self.scene.args]
    }
  }

  /// Asks for the counts recorded with the frame just submitted.
  pub fn request_stats(&self) {
    self.stats.request();
  }

  /// Clusters and triangles the latest counted frame drew.
  pub fn take_stats(&mut self) -> (u32, u32) {
    let [clusters, triangles, ..] = self.stats.take();

    (clusters, triangles)
  }

  /// What puts the level's finished scene into the window, once its targets are made.
  pub fn get_present_group(&self) -> Option<&wgpu::BindGroup> {
    self.light_groups.as_ref().map(|(_, groups)| &groups.present)
  }

  /// How far the level has loaded, when that changed since it was last asked.
  pub fn take_report(&mut self, textures: &TextureCache) -> Option<RenderLoadReport> {
    let slots = &self.scene.texture_slots;
    let (settled, total): (u32, u32) = (textures.count_settled(slots), slots.len() as u32);
    let sectors: u32 = self.scene.sectors.len() as u32 + self.failed;
    let report: RenderLoadReport = RenderLoadReport {
      sectors: self.scene.sectors.len() as u32,
      sectors_total: self.loader.get_total(),
      bytes: self.scene.get_bytes(),
      textures: settled,
      textures_total: total,
      is_ready: sectors == self.loader.get_total() && self.spawn.is_done() && settled == total,
    };

    if self.reported == Some(report) {
      return None;
    }

    self.reported = Some(report);

    Some(report)
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
