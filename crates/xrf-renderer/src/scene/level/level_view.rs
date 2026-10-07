use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, UVec4, Vec2, Vec3, Vec4};
use xrf_engine_target::XrayEngine;
use xrf_error::XrfResult;
use xrf_material::XraySurfaceDraw;
use xrf_math::EPS_S;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphRuntime,
  GraphTexture, GraphTextureAccess, ProxyHandle, RasterPassBuilder, StorageArray,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_ambient_occlusion_quality::RenderAmbientOcclusionQuality;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_applied_report::RenderAppliedReport;
use crate::contract::render_applied_shadows::RenderAppliedShadows;
use crate::contract::render_bloom_settings::RenderBloomSettings;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_level_hit::RenderLevelHit;
use crate::contract::render_lights_report::RenderLightsReport;
use crate::contract::render_load_report::RenderLoadReport;
use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_particles_report::RenderParticlesReport;
use crate::contract::render_pool_use::RenderPoolUse;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_selection::RenderSelection;
use crate::contract::render_selection_target::RenderSelectionTarget;
use crate::contract::render_static_report::RenderStaticReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_water_settings::RenderWaterSettings;
use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::fsr_targets::FsrTargets;
use crate::frame::pick_target::PickTarget;
use crate::frame::smaa_targets::SmaaTargets;
use crate::frame::smoothing_target::SmoothingTarget;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::temporal_history::TemporalHistory;
use crate::frame::temporal_jitter::TemporalJitter;
use crate::frame::upscale_targets::UpscaleTargets;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::host::render_rain::RenderRain;
use crate::host::render_sector_failure::RenderSectorFailure;
use crate::lighting::ambient_gust::AmbientGust;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::ambient_occlusion_pass::AmbientOcclusionPass;
use crate::pass::ambient_occlusion_uniform::AmbientOcclusionUniform;
use crate::pass::bloom_pass::BloomPass;
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
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;
use crate::pass::temporal_uniform::TemporalUniform;
use crate::pass::thunder_uniform::ThunderUniform;
use crate::pass::upscale_uniform::UpscaleUniform;
use crate::pass::view_binding::ViewBinding;
use crate::pass::view_light_groups::ViewLightGroups;
use crate::pass::water_draw::WaterDraw;
use crate::pass::wet_uniform::WetUniform;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::grass_level::GrassLevel;
use crate::scene::level::level_frame::LevelFrame;
use crate::scene::level::level_load::LevelLoad;
use crate::scene::level::level_overlays::LevelOverlays;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::level_smoothing::LevelSmoothing;
use crate::scene::level::level_water::WaterFrame;
use crate::scene::level::level_world_input::LevelWorldInput;
use crate::scene::level::lights_frame::LightsFrame;
use crate::scene::level::lights_view::LightsView;
use crate::scene::level::particle_emitter_proxy::ParticleEmitterProxy;
use crate::scene::level::placed_effect::PlacedEffect;
use crate::scene::level::scene_renderer::{SceneRenderer, SkyGroupKey};
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::shadow_sway::ShadowSway;
use crate::scene::level::view_info::ViewInfo;
use crate::scene::level::view_state::ViewState;
use crate::scene::level::weather_model_buffers::WeatherModelBuffers;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_layout::StaticLayout;
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

/// A level as one viewport draws it, held as the four things a frame is made from: the level (`LevelScene`), what the
/// view keeps between frames (`ViewState`), this frame as prepared (`ViewInfo`), and what draws it (`SceneRenderer`);
/// and what its load reports (`LevelLoad`). It orchestrates them: applies what the world posted, prepares the frame,
/// and records it as a frame graph.
pub struct LevelView {
  /// This frame as prepared, which its passes read.
  info: ViewInfo,
  /// What the view keeps from one frame to the next.
  state: ViewState,
  /// The level it draws.
  scene: LevelScene,
  /// What its load reports, from how far the world streamed it.
  load: LevelLoad,
  /// What draws it, and what that keeps between frames: the passes' buffers and bind groups, and the effects
  /// drawn from the view.
  renderer: SceneRenderer,
}

impl LevelView {
  /// The level it draws.
  pub fn get_scene(&self) -> &LevelScene {
    &self.scene
  }

  pub fn get_scene_mut(&mut self) -> &mut LevelScene {
    &mut self.scene
  }

  /// How far the level has loaded, when that changed since it was last asked.
  pub fn take_load_report(&mut self, textures: &TextureCache) -> Option<RenderLoadReport> {
    self.load.take_report(&self.scene, textures)
  }

  /// Whether everything the level opens with is resident, so it draws as it will.
  pub fn is_ready(&self, textures: &TextureCache) -> bool {
    self.load.is_ready(&self.scene, textures)
  }

  /// How far the level has loaded.
  pub fn describe_load(&self, textures: &TextureCache) -> RenderLoadReport {
    self.load.describe(&self.scene, textures)
  }

  /// The effects whose particles stopped playing this frame, for the world.
  pub fn take_finished_effects(&mut self) -> Vec<(ProxyHandle<ParticleEmitterProxy>, PlacedEffect)> {
    self.scene.particles.take_finished()
  }

  /// What its level's particles came to since the last report, as this view draws them.
  pub fn take_particles_report(&mut self) -> RenderParticlesReport {
    self.scene.particles.take_report(self.state.particles.drawn)
  }

  /// What it keeps from one frame to the next.
  pub fn get_state(&self) -> &ViewState {
    &self.state
  }

  pub fn get_state_mut(&mut self) -> &mut ViewState {
    &mut self.state
  }

  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    view_layout: &wgpu::BindGroupLayout,
    source: Arc<dyn RenderLevelSource>,
    workers: &RenderWorkers,
  ) -> Self {
    let scene: LevelScene = LevelScene::new(device, queue, view_layout, source, workers);
    let args_size: u64 = scene.statics.args.size();

    Self {
      info: ViewInfo::default(),
      state: ViewState::new(device, queue),
      load: LevelLoad::new(scene.started),
      scene,
      renderer: SceneRenderer::new(device, view_layout, args_size),
    }
  }

  /// Applies what the world posted this frame where it is the world's level, takes the grass and particles their
  /// loaders finished, and asks for the textures the lighting's sky draws with; answers the sectors it could not take
  /// in.
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
    world: Option<LevelWorldInput<'_>>,
  ) -> Vec<RenderSectorFailure> {
    let assets: Arc<dyn RenderAssetSource> = Arc::clone(&self.scene.source) as Arc<dyn RenderAssetSource>;

    if let Some(rain) = weather.and_then(|weather| weather.rain.as_ref())
      && lighting.rain.is_some()
      && options.mode.is_lit
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

    // Every bolt's textures are held while the level's weather has bolts, so a strike has them.
    if let Some(thunder) = weather.and_then(|weather| weather.thunder.as_ref())
      && options.mode.is_lit
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

    if options.mode.is_lit && options.show.is_sky_visible {
      self
        .renderer
        .flares
        .request((lighting, weather), weather_textures, &assets);
      weather_textures.request_sky(&lighting.sky, options.show.is_clouded, &assets);
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

    if let Some(slots) = self.scene.grass.poll(device, grass_pass, textures, &assets) {
      self.scene.statics.texture_slots.extend(slots);
    }

    let Some(LevelWorldInput {
      updates,
      streaming,
      skeleton_segments,
      gust,
      campfire_shares,
      motions,
    }) = world
    else {
      return Vec::new();
    };
    let failures: Vec<RenderSectorFailure> =
      self
        .scene
        .apply((device, queue, encoder), (&mut *textures, &assets), updates);

    self.load.advance(streaming, &self.scene, textures);
    self.info.skeleton_segments = skeleton_segments.to_vec();
    self.info.gust = gust;
    self.info.campfire_shares.clone_from(campfire_shares);
    self.info.motions.clone_from(motions);

    failures
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
    (view_layout, textures): (&wgpu::BindGroupLayout, &TextureCache),
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
      self.renderer.light_groups = None;
    }

    self.scene.statics.prepare_draws(device, queue, encoder);

    if let Some(overlays) = self.state.overlays.as_mut().filter(|it| it.skeleton.is_some()) {
      overlays.set_skeleton(device, &self.info.skeleton_segments);
    }

    self
      .renderer
      .shadows
      .prepare(device, options.features.shadows.resolution);

    let shadow_epoch: u64 = self.renderer.shadows.get_epoch();

    if self
      .renderer
      .light_groups
      .as_ref()
      .is_none_or(|(epoch, _)| *epoch != shadow_epoch)
      && let Some(targets) = &self.state.targets
    {
      let groups: ViewLightGroups = ViewLightGroups {
        sun: passes.sun.create_bind_group(
          device,
          targets,
          passes.table,
          &self.renderer.lighting,
          &self.renderer.shadows,
        ),
        lights: passes.lights.create_bind_groups(
          device,
          targets,
          passes.table,
          &self.state.lights.get_buffers(),
          self.scene.lights.get_shadow_atlas(),
        ),
        occlusion: passes
          .ambient_occlusion
          .create_bind_groups(device, targets, &self.renderer.occlusion_uniform),
        combine: passes.combine.create_bind_group(
          device,
          targets,
          passes.table,
          &self.renderer.lighting,
          &self.state.exposure.state,
        ),
        composited: passes.composited.create_bind_group(
          device,
          passes.table,
          (&self.renderer.lighting, &self.state.exposure.state),
          &self.renderer.shadows,
        ),
        haze: passes
          .sky_haze
          .create_bind_group(device, &self.renderer.lighting, &self.state.exposure.state),
        sun_shafts: passes.sun_shafts.create_bind_group(
          device,
          targets,
          &self.renderer.shadows,
          (&self.renderer.lighting, &self.state.exposure.state),
        ),
        exposure: passes.exposure.create_bind_group(device, targets, &self.state.exposure),
      };

      self.renderer.light_groups = Some((shadow_epoch, groups));
    }

    self
      .state
      .exposure
      .prepare(queue, &options.features.exposure, Instant::now());

    self.info.sun_sprite = self.renderer.flares.prepare(
      device,
      queue,
      passes.flares,
      (lighting, weather),
      options,
      weather_textures,
      (view, weather_rate),
      (
        self.state.targets.as_ref(),
        self.state.targets_epoch,
        &self.renderer.shadows,
      ),
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

    if self.renderer.sky_group.as_ref().is_none_or(|(key, _)| *key != sky_key) {
      self.renderer.sky_group = Some((
        sky_key,
        passes.sky.create_bind_group(
          device,
          weather_textures,
          (sky, self.info.sun_sprite.as_ref().map(|(texture, _)| texture.as_str())),
          &self.scene.environments.1,
        ),
      ));
    }

    // The ambient effects blow the wind the grass, the rain and the campfires read this frame.
    let gust: AmbientGust = self.info.gust;

    self.info.sky_cubes = Some([0, 1].map(|index| {
      weather_textures
        .get_view(sky.textures[index].as_deref(), WeatherTextureKind::Cube)
        .clone()
    }));
    self.state.water.prepare(
      device,
      options,
      WaterFrame {
        targets: self
          .state
          .targets
          .as_ref()
          .map(|targets| (targets, self.state.targets_epoch)),
        intensity: lighting.water_intensity,
        wind: lighting.wind,
        rain: lighting.rain.map_or(0.0, |rain| rain.density),
        time: self.scene.started.elapsed().as_secs_f32(),
      },
    );

    let sway_time: f32 = self.scene.started.elapsed().as_secs_f32();
    let wind: WindUniform =
      WindUniform::new(lighting.trees.as_ref(), sway_time).following(self.state.last_wind.as_ref());

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
    self.info.wind = wind;

    // A keyframe whose textures are not all up is blended out, so a sky still going up shows the other one.
    let side = |index: usize| {
      let clouds: Option<&str> = sky.clouds.textures[index]
        .as_deref()
        .filter(|_| options.show.is_clouded);

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

    self.info.is_hazing = options.mode.is_lit && options.show.is_sky_visible && options.show.is_sky_hazed;
    self.info.is_shafted = options.mode.is_lit
      && options.show.is_sun_shafted
      && lighting.get_sun_shafts(&options.features.sun_shafts) > 0.0
      && options.features.shadows.get_cascade_count() > 0;
    self.info.is_wallmarked = options.show.is_wallmarked;
    self.prepare_bloom(device, queue, passes, options, lighting.engine);

    if !options.features.is_occlusion_culled {
      self.state.history = None;
    }

    // The engine's screen: the viewport's pixels, widened for a lens narrower than its 90 degrees.
    let screen: f32 =
      (width * height) as f32 * (90.0 / field_of_view.max(1.0)).powi(2) * (EPS_S + options.features.lod.geometry_lod);
    let threshold = |area: f32| -> f32 { (area / 3.0).powi(2) / screen };

    self.info.cull = StaticCullParams {
      cluster_count: self.scene.statics.get_cluster_count(),
      row_count: self.scene.statics.get_row_count(),
      batch_count: StaticBatch::COUNT as u32,
      impostor_count: self.scene.statics.get_impostor_count(),
      glod_start: threshold(options.features.lod.ssa_glod_start),
      glod_end: threshold(options.features.lod.ssa_glod_end),
      discard_below: options.features.lod.ssa_discard.powi(2) / screen,
      candidate_capacity: self.scene.statics.get_list_capacity(),
      is_occluding: options.features.is_occlusion_culled as u32,
      lod_a: threshold(options.features.lod.ssa_a),
      lod_b: threshold(options.features.lod.ssa_b),
      is_impostors: options.features.lod.is_impostors as u32,
      pad: UVec4::ZERO,
      lod_origin: view.position.extend(1.0),
    };
    self.info.matrices = (view.view, view.projection);
    self.state.grass.prepare(
      (device, queue),
      (passes.grass, self.scene.grass.get_level()),
      &options.features.grass,
      view,
      self.info.cull.discard_below,
      (
        self.scene.started.elapsed().as_secs_f32(),
        lighting.trees.is_some(),
        gust.strength,
      ),
    );
    self.scene.lights.prepare(
      queue,
      &mut self.state.lights,
      LightsFrame {
        camera: view,
        settings: &options.features.lights,
        lod: (self.info.cull.glod_start, self.info.cull.glod_end),
        contents: self.scene.statics.get_contents(),
        sway: &to_sway(&self.scene.statics, self.info.sway),
        campfire_shares: &self.info.campfire_shares,
        motions: &self.info.motions,
      },
    );
    self.info.camera = *view;
    self.info.sun_direction = lighting.get_sun_direction();
    self.info.shadow_settings = options.features.shadows.clone();
    self.info.ambient_occlusion = options.features.ambient_occlusion;
    self.info.output = output;
    self.info.upscaling = options.output.upscaling;
    self.info.debug_view = options.mode.debug_view;
    self.info.is_wireframe = options.mode.is_wireframe;
    self.info.corrections = options.features.corrections;
    self.info.is_occlusion_drawn = options.mode.is_lit && options.features.ambient_occlusion.is_enabled;
    self.prepare_temporal(device, queue, passes, view);
    self.prepare_smoothing(device, passes, options.features.antialiasing);
    self.prepare_upscale(device, queue, passes);
    self.info.lights_settings = options.features.lights;

    queue.write_buffer(
      &self.renderer.occlusion_uniform,
      0,
      bytemuck::bytes_of(&AmbientOcclusionUniform::new(
        &options.features.ambient_occlusion,
        view.projection,
        (width.div_ceil(2), height.div_ceil(2)),
      )),
    );
    self.prepare_sorted(device, queue, view);

    if let Some((pyramid, _)) = &self.state.pyramid {
      let (history_view, history_projection): (Mat4, Mat4) = self.state.history.unwrap_or(self.info.matrices);

      self.info.occlusion = StaticOcclusionUniform {
        view: history_view,
        projection: history_projection,
        size: Vec2::new(pyramid.width as f32, pyramid.height as f32),
        levels: pyramid.levels,
        has_history: self.state.history.is_some() as u32,
      };
    }
    self.info.lighting = LightingUniform::new(lighting, view.view, options, &frame);
    queue.write_buffer(&self.renderer.lighting, 0, bytemuck::bytes_of(&self.info.lighting));

    self.scene.particles.simulate(view, options);
    self.scene.particles.fill(view, options, &mut self.state.particles);

    if let Some(targets) = &self.state.targets {
      self.state.particles.upload(
        (device, queue),
        passes.particles,
        (&self.renderer.lighting, self.scene.particles.get_surfaces()),
        (targets, self.state.targets_epoch),
      );
    }

    self.write_present(queue, options);
    self.prepare_shadows(device, queue, encoder, (view_layout, textures));
  }

  /// Readies this frame's shadows, sun, rain cover and lights, which the frame's passes then draw; `encoder` takes
  /// the copies a list growing makes.
  fn prepare_shadows(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    (view_layout, textures): (&wgpu::BindGroupLayout, &TextureCache),
  ) {
    let LevelView {
      scene, info, renderer, ..
    } = self;
    let SceneRenderer {
      shadows, rain_cover, ..
    } = renderer;
    let LevelScene { statics, lights, .. } = scene;
    let frame: ShadowFrame<'_> = ShadowFrame {
      scene: statics,
      camera: &info.camera,
      settings: &info.shadow_settings,
      sun_direction: info.sun_direction,
      sway: to_sway(statics, info.sway),
      params: &info.cull,
      textures,
    };

    shadows.prepare_cascades(device, queue, encoder, view_layout, &frame);

    if info.rain_draw.is_some() {
      rain_cover.prepare(device, queue, encoder, &frame);
    }

    lights.prepare_shadows(device, queue, encoder, &frame);
  }

  /// Declares the scene's culls and G-buffer draws: what last frame's depth does not hide, culled and drawn; then, while
  /// it culls occlusion, this frame's depth reduced and what the first draw does not hide of the rest culled and drawn.
  fn add_gbuffer_passes<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    passes: LevelPasses<'a>,
    (view, textures, scene): (&'a ViewBinding, &'a wgpu::BindGroup, &StaticSceneHandles),
    (targets, pyramid, pyramid_groups): (ViewTargetHandles, &'a DepthPyramid, &'a [wgpu::BindGroup]),
    is_occluding: bool,
  ) {
    let params: &'a StaticCullParams = &self.info.cull;
    let cull: StaticCullParameters = scene.get_camera_cull();
    let draws: StaticDraws = scene.get_camera_draws();
    let gbuffer: [GraphTexture; 4] = targets.get_gbuffer();
    let depth: GraphTexture = targets.depth;
    let add_draw = |graph: &mut FrameGraph<'a>, name: &'static str, draw_args: GraphBuffer, is_first: bool| {
      let color_load: wgpu::LoadOp<wgpu::Color> = if is_first {
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
      } else {
        wgpu::LoadOp::Load
      };
      let depth_load: wgpu::LoadOp<f32> = if is_first {
        wgpu::LoadOp::Clear(0.0)
      } else {
        wgpu::LoadOp::Load
      };
      let builder = gbuffer
        .iter()
        .fold(graph.add_raster_pass(name), |builder, texture| {
          builder.color(GraphColorAttachment::new(*texture, color_load))
        })
        .depth(GraphDepthAttachment::new(depth, depth_load))
        .buffer(draw_args, GraphBufferAccess::Indirect);

      draws.declare(builder, is_first).record(move |context| {
        let draw_args: &wgpu::Buffer = context.get_buffer(draw_args);

        passes
          .gbuffer
          .record(context, (view, &draws, textures), draw_args, is_first);
      });
    };

    graph
      .add_compute_pass("cull")
      .parameters(&cull)
      .record(move |context| passes.cull.record_early(context, view, &cull, params));
    add_draw(graph, "g-buffer", scene.args, true);

    if !is_occluding {
      return;
    }

    let (late, late_dispatch, reduced): (GraphBuffer, GraphBuffer, GraphTexture) =
      (scene.late, scene.late_dispatch, scene.pyramid);

    graph
      .add_compute_pass("depth pyramid")
      .texture(depth, GraphTextureAccess::Sampled)
      .texture(reduced, GraphTextureAccess::StorageReadWrite)
      .record(move |context| passes.pyramid.record(context.get_pass(), pyramid, pyramid_groups));
    graph
      .add_encoder_pass("late cull dispatch")
      .buffer(late, GraphBufferAccess::CopySource)
      .buffer(late_dispatch, GraphBufferAccess::CopyDestination)
      .record(move |context| {
        let (late, dispatch) = (context.get_buffer(late), context.get_buffer(late_dispatch));

        passes.cull.record_late_dispatch(context.get_encoder(), late, dispatch);
      });
    graph
      .add_compute_pass("late cull")
      .buffer(late_dispatch, GraphBufferAccess::Indirect)
      .parameters(&cull)
      .record(move |context| {
        let dispatch: &wgpu::Buffer = context.get_buffer(late_dispatch);

        passes.cull.record_late(context, view, &cull, dispatch);
      });
    add_draw(graph, "late g-buffer", late, false);
  }

  /// Declares the lighting: the lights binned into the view's clusters, the sun then every light drawn into the light
  /// target, the binning's overflow read back, and the ambient occlusion searched and denoised.
  fn add_lighting_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    (view, textures, groups): (&'a ViewBinding, &'a wgpu::BindGroup, &'a ViewLightGroups),
    targets: ViewTargetHandles,
    (has_lights, is_occlusion_ambient): (bool, bool),
  ) {
    let lights: &'a LightsView = &self.state.lights;
    let counts: GraphBuffer = bindings.import_buffer(graph, "light cluster counts", &lights.counts);
    let gbuffer: [GraphTexture; 4] = [targets.albedo, targets.normal, targets.material, targets.depth];

    graph
      .add_encoder_pass("light counts clear")
      .buffer(counts, GraphBufferAccess::CopyDestination)
      .record(move |context| lights.clear_overflow(context.get_encoder()));

    if has_lights {
      graph
        .add_compute_pass("light binning")
        .buffer(counts, GraphBufferAccess::StorageReadWrite)
        .record(move |context| passes.lights.record_binning(context.get_pass(), &groups.lights[0]));
    }

    gbuffer
      .into_iter()
      .fold(graph.add_raster_pass("sun"), |builder, texture| {
        builder.texture(texture, GraphTextureAccess::Sampled)
      })
      .color(GraphColorAttachment::new(
        targets.light,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .record(move |context| passes.sun.record(context.get_pass(), view, &groups.sun));

    if has_lights {
      gbuffer
        .into_iter()
        .fold(graph.add_raster_pass("lights"), |builder, texture| {
          builder.texture(texture, GraphTextureAccess::Sampled)
        })
        .buffer(counts, GraphBufferAccess::StorageRead)
        .color(GraphColorAttachment::new(targets.light, wgpu::LoadOp::Load))
        .record(move |context| {
          passes
            .lights
            .record_draw(context.get_pass(), view, &groups.lights, textures)
        });
    }

    // The overflow, read back for a report a frame or more later: an effect the graph cannot see.
    graph
      .add_encoder_pass("light overflow")
      .buffer(counts, GraphBufferAccess::CopySource)
      .keep()
      .record(move |context| lights.record_overflow(context.get_encoder()));

    if is_occlusion_ambient {
      let quality: RenderAmbientOcclusionQuality = self.info.ambient_occlusion.quality;

      for (stage, (name, (read, written))) in ["ambient occlusion", "occlusion denoise", "occlusion denoise back"]
        .into_iter()
        .zip(AmbientOcclusionPass::STAGES)
        .enumerate()
      {
        // Each stage reads the other target than it draws into, with the normals and the depth.
        [targets.occlusion[1 - read], targets.normal, targets.depth]
          .into_iter()
          .fold(graph.add_raster_pass(name), |builder, texture| {
            builder.texture(texture, GraphTextureAccess::Sampled)
          })
          .color(GraphColorAttachment::new(
            targets.occlusion[written],
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .record(move |context| {
            passes
              .ambient_occlusion
              .record(context.get_pass(), (stage, quality), view, &groups.occlusion)
          });
      }
    }
  }

  /// Adds a pass copying the whole of one texture into another of its size.
  fn add_copy(graph: &mut FrameGraph<'_>, name: &'static str, source: GraphTexture, destination: GraphTexture) {
    graph
      .add_encoder_pass(name)
      .texture(source, GraphTextureAccess::CopySource)
      .texture(destination, GraphTextureAccess::CopyDestination)
      .record(move |context| {
        let (source, destination) = (context.get_texture(source), context.get_texture(destination));

        context.get_encoder().copy_texture_to_texture(
          source.texture.as_image_copy(),
          destination.texture.as_image_copy(),
          source.texture.size(),
        );
      });
  }

  /// Adds a raster pass blending over the scene, tested against its depth without writing it.
  fn add_over_scene<'g, 'a>(
    graph: &'g mut FrameGraph<'a>,
    targets: ViewTargetHandles,
    name: &'static str,
  ) -> RasterPassBuilder<'g, 'a> {
    graph
      .add_raster_pass(name)
      .color(GraphColorAttachment::new(targets.scene, wgpu::LoadOp::Load))
      .depth(GraphDepthAttachment::new_read_only(targets.depth))
  }

  /// Ends a frame its graph recorded: the water's reflection and the temporal resolve's history it wrote become the
  /// ones the next frame keeps.
  pub fn end_frame(&mut self, frame: &LevelFrame) {
    if frame.is_lit {
      self.state.water.finish_frame();
    }

    match frame.resolve {
      Some("fsr2") => {
        if let Some((fsr, _)) = &mut self.state.fsr {
          fsr.swap();
        }
      }
      Some("temporal") => {
        if let Some((history, _)) = &mut self.state.temporal {
          history.swap();
        }
      }
      _ => {}
    }
  }

  /// Writes what the present pass reads, once the frame knows what draws into the distortion target: `def_distort`
  /// while the water or a particle does, nothing otherwise.
  fn write_present(&self, queue: &wgpu::Queue, options: &RenderViewOptions) {
    let Some(targets) = &self.state.targets else {
      return;
    };
    let water: &RenderWaterSettings = &options.features.water;
    let is_water_distorting: bool = water.is_enabled && water.is_distorted && options.mode.is_lit;
    let is_distorting: bool =
      !options.mode.is_wireframe && (is_water_distorting || self.state.particles.is_distorting());

    queue.write_buffer(
      &self.renderer.present,
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
    let bloom: &RenderBloomSettings = &options.features.bloom;

    self.info.is_bloomed =
      bloom.is_enabled && options.mode.is_lit && !options.mode.is_wireframe && self.state.targets.is_some();

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

    for (buffer, uniform) in self.renderer.bloom_uniforms.iter().zip(uniforms) {
      queue.write_buffer(buffer, 0, bytemuck::bytes_of(&uniform));
    }

    if self
      .renderer
      .bloom_groups
      .as_ref()
      .is_none_or(|(epoch, _)| *epoch != self.state.targets_epoch)
    {
      self.renderer.bloom_groups = Some((
        self.state.targets_epoch,
        passes
          .bloom
          .create_bind_groups(device, targets, &self.renderer.bloom_uniforms),
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
    let Some(rainfall) = lighting.rain.filter(|_| options.mode.is_lit) else {
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
      self.renderer.rain_cover.get_window(),
      self.scene.started.elapsed().as_secs_f32(),
      splash.index_count,
    );
    let group_key: (u64, usize) = (weather_textures.get_generation(), key);

    queue.write_buffer(&self.renderer.rain, 0, bytemuck::bytes_of(&uniform));

    if self
      .renderer
      .rain_group
      .as_ref()
      .is_none_or(|(built, _)| *built != group_key)
    {
      let flat = |reference: Option<&str>| weather_textures.get_view(reference, WeatherTextureKind::Flat);
      let group: wgpu::BindGroup = passes.rain.create_bind_group(
        device,
        &RainBindings {
          uniform: &self.renderer.rain,
          cover: &self.renderer.rain_cover.depth,
          streak: flat(Some(&rain.streak)),
          splash: flat(rain.drop.as_ref().map(|drop| drop.texture.as_str())),
          vertices: &splash.vertices,
          indices: &splash.indices,
        },
      );

      self.renderer.rain_group = Some((group_key, group));
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
      &self.renderer.wet,
      0,
      bytemuck::bytes_of(&WetUniform {
        density: rainfall.density.clamp(0.0, 1.0),
        time: uniform.time,
        is_extended: (lighting.engine == XrayEngine::Extended) as u32 as f32,
        pad: 0.0,
        window: uniform.window,
      }),
    );

    if self
      .renderer
      .wet_groups
      .as_ref()
      .is_none_or(|(built, _)| *built != wet_key)
    {
      let groups: [wgpu::BindGroup; 2] = passes.wet.create_bind_groups(
        device,
        targets,
        &self.renderer.rain_cover.depth,
        (
          weather_textures.get_view(Some(&wet.splash), WeatherTextureKind::Volume),
          weather_textures.get_view(Some(&wet.flow), WeatherTextureKind::Flat),
        ),
        &self.renderer.wet,
      );

      self.renderer.wet_groups = Some((wet_key, groups));
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
    let Some(thunder) = weather.thunder.as_ref().filter(|_| options.mode.is_lit) else {
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

    queue.write_buffer(
      &self.renderer.thunder,
      0,
      bytemuck::bytes_of(&ThunderUniform::new(strike)),
    );

    if self
      .renderer
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
          .create_bind_group(device, &self.renderer.thunder, flat(model_texture), buffers),
        passes.thunder.create_bind_group(
          device,
          &self.renderer.thunder,
          flat(&bolt.top.texture),
          &self.scene.no_model,
        ),
        passes.thunder.create_bind_group(
          device,
          &self.renderer.thunder,
          flat(&bolt.center.texture),
          &self.scene.no_model,
        ),
      ];

      self.renderer.thunder_groups = Some((group_key, groups));
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

  /// Readies a frame for its graph, none before the view's targets are made: a pick's camera narrowed to the texel
  /// picked and the readback it is copied into while one is free, and the depth the next frame's occlusion tests.
  pub fn begin_frame(
    &mut self,
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    pick: Option<(&CameraUniform, &wgpu::BindGroupLayout)>,
  ) -> Option<LevelFrame> {
    if self.state.targets.is_none() || self.state.pyramid.is_none() {
      return None;
    }

    let pick_slot: Option<usize> =
      pick.and_then(|(camera, view_layout)| self.ready_pick(device, queue, camera, view_layout));

    if self.info.cull.is_occluding != 0 {
      self.state.history = Some(self.info.matrices);
    }

    Some(LevelFrame {
      pick_slot,
      is_lit: self.renderer.light_groups.is_some(),
      resolve: if self.state.fsr.is_some() {
        Some("fsr2")
      } else if self.state.temporal.is_some() {
        Some("temporal")
      } else if self.state.upscale.is_some() {
        Some("upscale")
      } else {
        None
      },
    })
  }

  /// Declares the frame `begin_frame` readied into the frame's graph: culls the scene and draws it into the G-buffer
  /// (what last frame's depth does not hide, then, culling occlusion, what this frame's first draw does not hide of the
  /// rest, leaving this frame's depth reduced for the next), shadows and lights it, draws the water and what blends over
  /// it, resolves the frame, and draws the pick last. Which passes run is decided here; each reads the view, which
  /// recording leaves as it is. The present pass reads what it leaves.
  pub fn record<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    textures: &'a TextureCache,
    frame: &LevelFrame,
  ) {
    let is_occluding: bool = self.info.cull.is_occluding != 0;
    let pick_slot: Option<usize> = frame.pick_slot;
    let resolve: Option<&'static str> = frame.resolve;
    let is_drawn: bool = !self.info.is_wireframe;
    let is_wallmarked: bool = self.info.is_wallmarked && is_drawn;
    let is_raining: bool = self.info.rain_draw.is_some();
    let is_wet: bool = is_raining && self.renderer.wet_groups.is_some();
    let is_lit: bool = frame.is_lit;
    let has_lights: bool = self.state.lights.get_count() > 0;
    let is_occlusion_ambient: bool = self.info.ambient_occlusion.is_enabled;
    let has_sky: bool = self.renderer.sky_group.is_some();
    let is_hazing: bool = has_sky && self.info.is_hazing;
    let is_composited: bool = is_drawn && has_sky;
    let has_particles: bool = is_drawn && self.state.particles.is_drawing();
    let is_shafted: bool = self.info.is_shafted;
    let is_rain_drawn: bool = is_raining && self.renderer.rain_group.is_some();
    let is_thundering: bool = self.info.thunder_draw.is_some() && self.renderer.thunder_groups.is_some();
    let is_bloomed: bool = self.info.is_bloomed && self.renderer.bloom_groups.is_some();
    let is_smoothed: bool = self.state.smoothing.is_some();
    let is_sharpened: bool = self.state.upscale.is_some() && self.info.upscaling.is_sharpened();
    let is_adapting: bool = self.state.exposure.is_adapting();
    let texture_group: &wgpu::BindGroup = textures.get_bind_group();
    let level_view: &'a LevelView = self;
    let (Some(targets), Some((pyramid, pyramid_groups))) = (&level_view.state.targets, &level_view.state.pyramid)
    else {
      return;
    };
    let handles: ViewTargetHandles = ViewTargetHandles::import(&mut *graph, &mut *bindings, targets);
    let statics: StaticSceneHandles = StaticSceneHandles::import(
      (&mut *graph, &mut *bindings, runtime),
      &level_view.scene.statics,
      &pyramid.view,
      (&level_view.info.cull, &level_view.info.occlusion, &level_view.info.wind),
    );
    let layouts: [StaticDrawParameters; StaticLayout::COUNT] = statics.get_camera_draws().layouts;

    // The frame is encoded in four groups, on as many threads where it is not timed: the scene, its shadows, its
    // lighting and water, and what blends over it and resolves it.
    graph.begin_group("scene");

    graph
      .add_raster_pass("distortion clear")
      .color(GraphColorAttachment::new(
        handles.distortion,
        wgpu::LoadOp::Clear(ViewTargets::DISTORTION_CLEAR),
      ))
      .record(|_| {});

    let grass_level: Option<&GrassLevel> = level_view.scene.grass.get_level();
    let grass_args: Option<GraphBuffer> =
      level_view
        .state
        .grass
        .add_planting(&mut *graph, &mut *bindings, (passes.grass, grass_level));

    level_view.add_gbuffer_passes(
      &mut *graph,
      passes,
      (view, texture_group, &statics),
      (handles, pyramid, pyramid_groups),
      is_occluding,
    );

    let stats_args: GraphBuffer = statics.args;

    // The cull's counts, read back for a report a frame or more later: an effect the graph cannot see.
    graph
      .add_encoder_pass("stats")
      .buffer(stats_args, GraphBufferAccess::CopySource)
      .keep()
      .record(move |context| {
        let args: &wgpu::Buffer = context.get_buffer(stats_args);

        level_view
          .state
          .stats
          .record(context.get_encoder(), args, StaticScene::STATS_OFFSET);
      });

    if let (true, Some(grass_args)) = (is_drawn, grass_args) {
      level_view.state.grass.add_draw(
        &mut *graph,
        (passes.grass, grass_level),
        (handles, grass_args),
        (view, texture_group),
      );
    }

    if is_wallmarked {
      let args: Vec<GraphBuffer> = Self::list_draw_args(&statics, &level_view.info.cull);

      StaticDraws::declare_layouts(
        args
          .iter()
          .fold(graph.add_raster_pass("wall marks"), |builder, args| {
            builder.buffer(*args, GraphBufferAccess::Indirect)
          })
          .color(GraphColorAttachment::new(handles.albedo, wgpu::LoadOp::Load))
          .depth(GraphDepthAttachment::new_read_only(handles.depth)),
        &layouts,
      )
      .record(move |context| {
        let args: Vec<&wgpu::Buffer> = args.iter().map(|args| context.get_buffer(*args)).collect();

        passes
          .composited
          .record_wallmarks(context, (view, &layouts, texture_group), &args);
      });
    }

    graph.begin_group("shadows");

    level_view.renderer.shadows.add_passes(
      (&mut *graph, &mut *bindings),
      passes,
      &statics,
      (&level_view.info.cull, texture_group),
    );

    if is_raining {
      level_view.renderer.rain_cover.add_passes(
        (&mut *graph, &mut *bindings),
        passes,
        &statics,
        (&level_view.info.cull, texture_group),
      );
    }

    level_view.scene.lights.add_shadow_passes(
      (&mut *graph, &mut *bindings),
      passes,
      &statics,
      (&level_view.info.cull, texture_group),
    );

    graph.begin_group("lighting");

    // The rain wets the G-buffer before any light is drawn over it.
    if let (true, Some((_, [patch, apply]))) = (is_wet, &level_view.renderer.wet_groups) {
      // The patches go into the light, which the wet look over the normals and the albedo then reads.
      for (stage, (name, target, group, reads)) in [
        (
          "wet patch",
          handles.light,
          patch,
          [handles.depth, handles.albedo, handles.normal],
        ),
        (
          "wet normal",
          handles.normal,
          apply,
          [handles.depth, handles.light, handles.light],
        ),
        (
          "wet albedo",
          handles.albedo,
          apply,
          [handles.depth, handles.light, handles.light],
        ),
      ]
      .into_iter()
      .enumerate()
      {
        reads
          .into_iter()
          .fold(graph.add_raster_pass(name), |builder, texture| {
            builder.texture(texture, GraphTextureAccess::Sampled)
          })
          .color(GraphColorAttachment::new(target, wgpu::LoadOp::Load))
          .record(move |context| passes.wet.record(context.get_pass(), stage, view, group));
      }
    }

    if let (true, Some((_, groups))) = (is_lit, &level_view.renderer.light_groups) {
      level_view.add_lighting_passes(
        (&mut *graph, &mut *bindings),
        passes,
        (view, texture_group, groups),
        handles,
        (has_lights, is_occlusion_ambient),
      );

      if let (true, Some((_, sky_group))) = (is_hazing, &level_view.renderer.sky_group) {
        graph
          .add_raster_pass("haze")
          .color(GraphColorAttachment::new(
            handles.haze,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .record(move |context| passes.sky_haze.record(context.get_pass(), &groups.haze, sky_group));
      }

      if let (true, Some((_, sky_group))) = (has_sky, &level_view.renderer.sky_group) {
        [
          handles.albedo,
          handles.normal,
          handles.material,
          handles.depth,
          handles.light,
          handles.occlusion[0],
          handles.haze,
        ]
        .into_iter()
        .fold(graph.add_raster_pass("combine"), |builder, texture| {
          builder.texture(texture, GraphTextureAccess::Sampled)
        })
        .color(GraphColorAttachment::new(
          handles.scene,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .color(GraphColorAttachment::new(
          handles.high,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .record(move |context| {
          passes
            .combine
            .record(context.get_pass(), view, &groups.combine, sky_group)
        });
      }

      let scene: GraphTexture = handles.scene;

      // FSR 2's reactive mask is what the water and the blended surfaces change of the frame drawn so far.
      if let Some((fsr, _)) = &level_view.state.fsr {
        let opaque: GraphTexture = bindings.import_view(&mut *graph, "fsr2 opaque", &fsr.opaque);

        graph
          .add_encoder_pass("fsr2 opaque")
          .texture(scene, GraphTextureAccess::CopySource)
          .texture(opaque, GraphTextureAccess::CopyDestination)
          .record(move |context| {
            let (scene, opaque) = (context.get_texture(scene), context.get_texture(opaque));

            context.get_encoder().copy_texture_to_texture(
              scene.texture.as_image_copy(),
              opaque.texture.as_image_copy(),
              opaque.texture.size(),
            );
          });
      }

      if let (true, Some(skies)) = (level_view.state.water.is_drawn(), &level_view.info.sky_cubes) {
        let draw: WaterDraw<'_> = WaterDraw {
          targets: handles,
          view,
          textures: texture_group,
          layouts,
          args: Self::list_draw_args(&statics, &level_view.info.cull),
          lighting: runtime.push_uniform(&level_view.info.lighting),
          skies: [&skies[0], &skies[1]],
          sky_sampler: passes.sky.get_clamp(),
          water: &level_view.state.water,
        };

        passes.water.add_passes(&mut *graph, &mut *bindings, runtime, draw);
      }

      graph.begin_group("post");

      if let (true, Some((_, sky_group))) = (is_composited, &level_view.renderer.sky_group) {
        let args: Vec<GraphBuffer> = Self::list_draw_args(&statics, &level_view.info.cull);
        // The models' composited clusters, back to front, which the view lists itself.
        let sorted: Option<StaticDrawParameters> = level_view
          .state
          .sorted_list
          .as_ref()
          .filter(|_| level_view.info.sorted_count > 0)
          .map(|list| {
            let list: GraphBuffer = bindings.import_buffer(&mut *graph, "sorted composited", list);

            statics.get_layout_draws(StorageArray::new(list))[StaticLayout::Model.get_index()]
          });
        let sorted_count: u32 = level_view.info.sorted_count;
        let builder = args.iter().fold(
          StaticDraws::declare_layouts(Self::add_over_scene(&mut *graph, handles, "composited"), &layouts),
          |builder, args| builder.buffer(*args, GraphBufferAccess::Indirect),
        );
        let builder = match &sorted {
          Some(sorted) => builder.parameters(sorted),
          None => builder,
        };

        builder.record(move |context| {
          let args: Vec<&wgpu::Buffer> = args.iter().map(|args| context.get_buffer(*args)).collect();

          passes.composited.record(
            context,
            (view, &layouts, texture_group),
            (&groups.composited, sky_group),
            &args,
            (sorted.as_ref(), sorted_count),
          );
        });
      }

      if has_particles {
        level_view
          .state
          .particles
          .add_passes(&mut *graph, passes.particles, handles, (view, texture_group));
      }

      if is_shafted {
        graph
          .add_raster_pass("sun shafts")
          .texture(handles.depth, GraphTextureAccess::Sampled)
          .color(GraphColorAttachment::new(handles.scene, wgpu::LoadOp::Load))
          .color(GraphColorAttachment::new(handles.high, wgpu::LoadOp::Load))
          .record(move |context| passes.sun_shafts.record(context.get_pass(), view, &groups.sun_shafts));
      }

      if let (true, Some(counts), Some((_, rain_group))) = (
        is_rain_drawn,
        level_view.info.rain_draw,
        &level_view.renderer.rain_group,
      ) {
        Self::add_over_scene(&mut *graph, handles, "rain")
          .record(move |context| passes.rain.record(context.get_pass(), view, rain_group, counts));
      }

      if let (true, Some(draws), Some((_, thunder_groups))) = (
        is_thundering,
        level_view.info.thunder_draw,
        &level_view.renderer.thunder_groups,
      ) {
        Self::add_over_scene(&mut *graph, handles, "thunder")
          .record(move |context| passes.thunder.record(context.get_pass(), view, thunder_groups, draws));
      }

      level_view
        .renderer
        .flares
        .add_passes(&mut *graph, passes.flares, handles, view);

      if let (true, Some((_, bloom_groups))) = (is_bloomed, &level_view.renderer.bloom_groups) {
        // Built from the high target into the first, blurred across into the second, then down into the first.
        for (stage, (read, written)) in [
          (handles.high, handles.bloom[0]),
          (handles.bloom[0], handles.bloom[1]),
          (handles.bloom[1], handles.bloom[0]),
        ]
        .into_iter()
        .enumerate()
        {
          graph
            .add_raster_pass(BloomPass::STAGES[stage])
            .texture(read, GraphTextureAccess::Sampled)
            .color(GraphColorAttachment::new(
              written,
              wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            ))
            .record(move |context| passes.bloom.record(context.get_pass(), stage, bloom_groups));
        }
      }

      if let Some(smoothing) = level_view.state.smoothing.as_ref().filter(|_| is_smoothed) {
        let target: GraphTexture = bindings.import_view(&mut *graph, "smoothed", &smoothing.target.view);

        match (&smoothing.smaa, passes.smaa) {
          (Some(smaa), Some(pass)) => {
            let edges: GraphTexture = bindings.import_view(&mut *graph, "smaa edges", &smaa.edges);
            let weights: GraphTexture = bindings.import_view(&mut *graph, "smaa weights", &smaa.weights);

            for (stage, (name, reads, written)) in [
              ("smaa edges", vec![handles.scene], edges),
              ("smaa weights", vec![edges], weights),
              ("smaa", vec![handles.scene, weights], target),
            ]
            .into_iter()
            .enumerate()
            {
              reads
                .into_iter()
                .fold(graph.add_raster_pass(name), |builder, texture| {
                  builder.texture(texture, GraphTextureAccess::Sampled)
                })
                .color(GraphColorAttachment::new(
                  written,
                  wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                ))
                .record(move |context| pass.record(context.get_pass(), stage, &smoothing.groups));
            }
          }
          _ => {
            graph
              .add_raster_pass("fxaa")
              .texture(handles.scene, GraphTextureAccess::Sampled)
              .color(GraphColorAttachment::new(
                target,
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
              ))
              .record(move |context| passes.fxaa.record(context.get_pass(), &smoothing.groups[0]));
          }
        }

        Self::add_copy(&mut *graph, "smoothed copy", target, handles.scene);
      }

      // The resolved frame goes where the present pass reads it: the upscaled frame, or the scene drawn at its size.
      let upscaled: Option<[GraphTexture; 2]> = level_view.state.upscale.as_ref().map(|(upscale, _)| {
        [
          bindings.import_view(&mut *graph, "upscaled", &upscale.views[0]),
          bindings.import_view(&mut *graph, "sharpened", &upscale.views[1]),
        ]
      });
      let resolved: GraphTexture = upscaled.map_or(handles.scene, |[upscaled, _]| upscaled);

      match (
        resolve,
        &level_view.state.fsr,
        &level_view.state.temporal,
        &level_view.state.upscale,
      ) {
        (Some("fsr2"), Some((fsr, groups)), _, _) => {
          let history: GraphTexture = bindings.import_view(&mut *graph, "fsr2 history", &fsr.history[fsr.index]);

          passes.fsr.add_passes(&mut *graph, &mut *bindings, (fsr, groups));
          Self::add_copy(&mut *graph, "fsr2 output", history, resolved);
        }
        (Some("temporal"), _, Some((history, groups)), _) => {
          let index: usize = history.index;
          let target: GraphTexture = bindings.import_view(&mut *graph, "temporal history", &history.views[index]);

          graph
            .add_raster_pass("temporal")
            .texture(handles.scene, GraphTextureAccess::Sampled)
            .color(GraphColorAttachment::new(
              target,
              wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            ))
            .record(move |context| passes.temporal.record(context.get_pass(), view, &groups[index]));
          Self::add_copy(&mut *graph, "temporal output", target, resolved);
        }
        (Some(_), _, _, Some((_, groups))) => {
          if let Some([upscaled, _]) = upscaled {
            graph
              .add_raster_pass("upscale")
              .texture(handles.scene, GraphTextureAccess::Sampled)
              .color(GraphColorAttachment::new(
                upscaled,
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
              ))
              .record(move |context| passes.upscale.record(context.get_pass(), 0, &groups[0]));
          }
        }
        _ => {}
      }

      if let (true, Some((_, groups)), Some([upscaled, sharpened])) =
        (is_sharpened, &level_view.state.upscale, upscaled)
      {
        graph
          .add_raster_pass("sharpen")
          .texture(upscaled, GraphTextureAccess::Sampled)
          .color(GraphColorAttachment::new(
            sharpened,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .record(move |context| passes.upscale.record(context.get_pass(), 1, &groups[1]));
      }

      if is_adapting {
        let state: GraphBuffer = bindings.import_buffer(&mut *graph, "exposure", &level_view.state.exposure.state);

        graph
          .add_compute_pass("exposure")
          .texture(resolved, GraphTextureAccess::Sampled)
          .buffer(state, GraphBufferAccess::StorageReadWrite)
          .record(move |context| passes.exposure.record(context.get_pass(), &groups.exposure));
      }
    }

    if let (Some(slot), Some(target), Some(pick_view)) =
      (pick_slot, &level_view.state.pick_target, &level_view.state.pick_view)
    {
      level_view.add_pick_passes(
        (&mut *graph, &mut *bindings),
        passes.gbuffer,
        (pick_view, texture_group, &statics),
        (target, slot),
      );
    }
  }

  /// The draw arguments a forward pass replays: the early phase's, and the late phase's where occlusion culls.
  fn list_draw_args(scene: &StaticSceneHandles, params: &StaticCullParams) -> Vec<GraphBuffer> {
    if params.is_occluding != 0 {
      vec![scene.args, scene.late]
    } else {
      vec![scene.args]
    }
  }

  /// Readies a pick's camera and target, and answers the readback it will be copied into; none while every readback is
  /// on its way.
  fn ready_pick(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    camera: &CameraUniform,
    view_layout: &wgpu::BindGroupLayout,
  ) -> Option<usize> {
    let slot: usize = self
      .state
      .pick_target
      .get_or_insert_with(|| PickTarget::new(device))
      .find_free_readback()?;

    self
      .state
      .pick_view
      .get_or_insert_with(|| ViewBinding::new(device, view_layout))
      .write(queue, camera);

    Some(slot)
  }

  /// Declares a pick: the frame's visible clusters drawn again into its texel, through its narrowed camera, after every
  /// cull of the frame, and the texel copied out into its readback.
  fn add_pick_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    gbuffer: &'a StaticGBufferPass,
    (view, textures, scene): (&'a ViewBinding, &'a wgpu::BindGroup, &StaticSceneHandles),
    (target, slot): (&'a PickTarget, usize),
  ) {
    let color: GraphTexture = bindings.import_view(graph, "pick", &target.color);
    let depth: GraphTexture = bindings.import_view(graph, "pick depth", &target.depth);
    let draws: StaticDraws = scene.get_camera_draws();
    let args: [GraphBuffer; 2] = [scene.args, scene.late];

    draws
      .declare(
        graph
          .add_raster_pass("static pick")
          .color(GraphColorAttachment::new(
            color,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .depth(GraphDepthAttachment::new(depth, wgpu::LoadOp::Clear(0.0)))
          .buffer(args[0], GraphBufferAccess::Indirect)
          .buffer(args[1], GraphBufferAccess::Indirect),
        true,
      )
      .record(move |context| {
        let args: [&wgpu::Buffer; 2] = args.map(|args| context.get_buffer(args));

        gbuffer.pick(context, (view, &draws, textures), &args);
      });
    graph
      .add_encoder_pass("pick copy")
      .texture(color, GraphTextureAccess::CopySource)
      .keep()
      .record(move |context| target.copy_out(context.get_encoder(), slot));
  }

  /// Asks for a pick's texel back, its frame just submitted.
  pub fn request_pick(&self, slot: usize) {
    if let Some(target) = &self.state.pick_target {
      target.request(slot);
    }
  }

  /// A pick's texel once it is back; a view made since the pick has no readback for it, and it met nothing still drawn.
  pub fn take_pick(&self, slot: usize) -> Option<XrfResult<[u32; 4]>> {
    match &self.state.pick_target {
      Some(target) => target.take(slot),
      None => Some(Ok([0; 4])),
    }
  }

  /// Names what a pick's texel met.
  pub fn resolve_pick(
    &self,
    [kind, cluster, place, depth]: [u32; 4],
    unproject: impl Fn(f32) -> Vec3,
  ) -> Option<RenderLevelHit> {
    let point: [f32; 3] = unproject(f32::from_bits(depth)).to_array();

    match kind {
      PICKED_CLUSTER => {}
      PICKED_IMPOSTOR => {
        return self
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
          });
      }
      _ => return None,
    }

    let (info, instance) = self.scene.statics.resolve_pick(cluster, place)?;

    if info.sector == StaticSlotInfo::NO_SECTOR {
      return self
        .scene
        .statics
        .resolve_spawn_pick(place)
        .map(|object| RenderLevelHit::Spawn { object, point });
    }

    Some(RenderLevelHit::Surface {
      sector: info.sector,
      shader_id: info.shader_id as u32,
      mesh: info.mesh,
      place: instance,
      is_impostor: false,
      point,
    })
  }

  /// Where the next frame's samples sit within their pixels, in drawn pixels, `y` down: jittered while it resolves
  /// frames temporally and shows the finished frame, still otherwise.
  pub fn next_jitter(&mut self, options: &RenderViewOptions, ratio: f32) -> Vec2 {
    self.info.is_temporal =
      options.features.antialiasing.is_temporal() && options.mode.debug_view == RenderDebugView::Final;
    self.info.is_fsr = self.info.is_temporal && options.features.antialiasing == RenderAntialiasing::Fsr2;
    self.info.jitter_phases = TemporalJitter::get_phases(ratio);
    self.info.jitter = if self.info.is_temporal {
      self.state.jitter.next(ratio)
    } else {
      Vec2::ZERO
    };

    self.info.jitter
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
          .create_bind_groups(device, targets, &history, &self.renderer.temporal_uniform);

      (history, groups)
    });
    let current: Mat4 = view.get_view_projection();
    let (previous, previous_view): (Mat4, Mat4) = self.state.temporal_previous.unwrap_or((current, view.view));

    queue.write_buffer(
      &self.renderer.temporal_uniform,
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
      let groups: FsrGroups = passes
        .fsr
        .create_bind_groups(device, targets, &fsr, &self.renderer.fsr_uniform);

      (fsr, groups)
    });

    queue.write_buffer(
      &self.renderer.fsr_uniform,
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
  fn prepare_sorted(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &CameraView) {
    let planes: [Vec4; 6] = view.get_planes();
    let mut places: Vec<(f32, StaticSortedPlace<'_>)> = self
      .scene
      .statics
      .list_sorted_places()
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
    }

    let Some(buffer) = &self.state.sorted_list else {
      return;
    };

    queue.write_buffer(buffer, 0, bytes);
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
          .create_bind_group(device, &targets.scene, &self.renderer.upscale_uniform),
        passes
          .upscale
          .create_bind_group(device, &upscale.views[0], &self.renderer.upscale_uniform),
      ];

      self.state.upscale = Some((upscale, groups));
      self.state.upscale_epoch += 1;
    }

    // An upscale made before the targets were reads a scene since dropped.
    if let Some((upscale, groups)) = &mut self.state.upscale
      && self
        .renderer
        .present_group
        .as_ref()
        .is_none_or(|(key, _)| key.0 != self.state.targets_epoch)
    {
      groups[0] = passes
        .upscale
        .create_bind_group(device, &targets.scene, &self.renderer.upscale_uniform);
      groups[1] = passes
        .upscale
        .create_bind_group(device, &upscale.views[0], &self.renderer.upscale_uniform);
    }

    queue.write_buffer(
      &self.renderer.upscale_uniform,
      0,
      bytemuck::bytes_of(&UpscaleUniform {
        output_size: [output.width as f32, output.height as f32],
        sharpness: self.info.upscaling.get_sharpness(),
        pad: 0.0,
      }),
    );
    if self
      .renderer
      .overlay_group
      .as_ref()
      .is_none_or(|(epoch, _)| *epoch != self.state.targets_epoch)
    {
      let group: wgpu::BindGroup =
        passes
          .overlay
          .create_bind_group(device, targets, &self.renderer.present, &self.renderer.lighting);

      self.renderer.overlay_group = Some((self.state.targets_epoch, group));
    }

    let shown: usize = usize::from(self.info.upscaling.is_sharpened());
    let key: (u64, u64, usize) = (
      self.state.targets_epoch,
      if is_upscaled { self.state.upscale_epoch } else { 0 },
      shown,
    );

    if self.renderer.present_group.as_ref().is_none_or(|(it, _)| *it != key) {
      let upscaled: Option<&wgpu::TextureView> = self.state.upscale.as_ref().map(|(upscale, _)| &upscale.views[shown]);
      let group: wgpu::BindGroup = passes
        .present
        .create_bind_group(device, targets, &self.renderer.present, upscaled);

      self.renderer.present_group = Some((key, group));
    }
  }

  /// Asks for the counts recorded with the frame just submitted.
  pub fn request_stats(&self) {
    self.state.stats.request();
    self.state.lights.request_report();
  }

  /// What its frames are drawn with, as resolved from what `options` asked; the weather's light is the viewport's to add.
  pub fn describe_applied(&self, options: &RenderViewOptions) -> RenderAppliedReport {
    let antialiasing: RenderAntialiasing = if self.info.is_temporal {
      options.features.antialiasing
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
        resolution: self.renderer.shadows.get_maps().resolution,
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
      grass: self.state.grass.get_applied(&options.features.grass),
      is_water: options.features.water.is_enabled,
      environment: None,
      sun: self
        .info
        .sun_sprite
        .as_ref()
        .and_then(|_| self.renderer.flares.get_shown())
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
      self.state.lights.take_report(),
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
    let contents: usize = self.scene.statics.get_contents();

    if self
      .state
      .selection
      .as_ref()
      .is_none_or(|(target, at, _)| *target != selection.target || *at != contents)
    {
      self.state.selection = Some((
        selection.target,
        contents,
        self.scene.statics.resolve_selection(&selection.target),
      ));
    }

    let resolved: &StaticSelection = self.state.selection.as_ref()?.2.as_ref()?;

    self.info.selection_color = Some(selection.color);

    Some(resolved)
  }

  /// What it draws over its frame, with the bind group drawing it, once both are made.
  pub fn get_overlays(&self) -> Option<(&wgpu::BindGroup, &LevelOverlays)> {
    Some((&self.renderer.overlay_group.as_ref()?.1, self.state.overlays.as_ref()?))
  }

  /// What puts the level's finished scene into the window, once its targets are made.
  pub fn get_present_group(&self) -> Option<&wgpu::BindGroup> {
    self.renderer.present_group.as_ref().map(|(_, group)| group)
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
