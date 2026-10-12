use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, UVec4, Vec2, Vec3, Vec4};
use xrf_error::XrfResult;
use xrf_math::EPS_S;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphRuntime,
  GraphTexture, GraphTextureAccess, GraphTextureDescriptor, StorageArray, StorageArrayMut, StorageValue,
  UniformBinding,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_applied_indirect_light::RenderAppliedIndirectLight;
use crate::contract::render_applied_reflections::RenderAppliedReflections;
use crate::contract::render_applied_report::RenderAppliedReport;
use crate::contract::render_applied_shadows::RenderAppliedShadows;
use crate::contract::render_bloom_mode::RenderBloomMode;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_foliage_settings::RenderFoliageSettings;
use crate::contract::render_level_hit::RenderLevelHit;
use crate::contract::render_lights_report::RenderLightsReport;
use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_particles_report::RenderParticlesReport;
use crate::contract::render_pool_use::RenderPoolUse;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_reflection_probe::RenderReflectionProbe;
use crate::contract::render_selection::RenderSelection;
use crate::contract::render_selection_target::RenderSelectionTarget;
use crate::contract::render_static_report::RenderStaticReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_water_settings::RenderWaterSettings;
use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::pick_target::PickTarget;
use crate::frame::sky_prefilter::SkyPrefilter;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::texture_copy::add_texture_copy;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::lighting::ambient_gust::AmbientGust;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::combine_parameters::CombineParameters;
use crate::pass::composited_parameters::CompositedParameters;
use crate::pass::contact_shadow_uniform::ContactShadowUniform;
use crate::pass::foliage_wind_values::FoliageWindValues;
use crate::pass::froxel_uniform::FroxelUniform;
use crate::pass::grass_colour_parameters::GrassColourParameters;
use crate::pass::grass_puddles_uniform::GrassPuddlesUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::light_binning_parameters::LightBinningParameters;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::lights_parameters::LightsParameters;
use crate::pass::lights_uniform::LightsUniform;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::pyramid_depth_parameters::PyramidDepthParameters;
use crate::pass::pyramid_level_parameters::PyramidLevelParameters;
use crate::pass::rain_uniform::RainUniform;
use crate::pass::reflection_trace::ReflectionTrace;
use crate::pass::sky_haze_parameters::SkyHazeParameters;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::sky_prefilter_parameters::SkyPrefilterParameters;
use crate::pass::sky_prefilter_pass::SkyPrefilterPass;
use crate::pass::sky_prefilter_uniform::SkyPrefilterUniform;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;
use crate::pass::sun_parameters::SunParameters;
use crate::pass::sun_shafts_parameters::SunShaftsParameters;
use crate::pass::sun_shafts_pass::SunShaftsPass;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::view_binding::ViewBinding;
use crate::pass::water_draw::WaterDraw;
use crate::pass::water_uniform::WaterUniform;
use crate::pass::wet_uniform::WetUniform;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::cover_textures::CoverTextures;
use crate::scene::level::feature::composited_draw::CompositedDraw;
use crate::scene::level::feature::overhead_windows::OverheadWindows;
use crate::scene::level::feature::scene_features::SceneFeatures;
use crate::scene::level::feature::water_frame::WaterFrame;
use crate::scene::level::frame_load::FrameLoad;
use crate::scene::level::grass_level::GrassLevel;
use crate::scene::level::level_frame::LevelFrame;
use crate::scene::level::level_overlays::LevelOverlays;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::level_surface::{LevelSurface, PuddleKeepInputs};
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::level::lights_frame::LightsFrame;
use crate::scene::level::lights_view::LightsView;
use crate::scene::level::scene_output::SceneOutput;
use crate::scene::level::scene_renderer::SceneRenderer;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::shadow_sway::ShadowSway;
use crate::scene::level::surroundings_cube::SurroundingsCube;
use crate::scene::level::view_info::ViewInfo;
use crate::scene::level::view_state::ViewState;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_cull_scratch::StaticCullScratch;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::static_scene::static_selection::StaticSelection;
use crate::scene::static_scene::static_sight::StaticSight;
use crate::scene::static_scene::static_slot_info::StaticSlotInfo;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// The sway amplitude the shadows are redrawn at while the enhanced foliage motion blows: its largest move a metre of
/// reach, a tall crown's trunk swing over its height.
const FOLIAGE_SHADOW_AMPLITUDE: f32 = 0.02;

/// The camera hemi under which the enhanced fog's height fog is gone and over which it is whole, about the engine's
/// indoors at `0.05` (`CGamePersistent::WeathersUpdate`).
const INDOOR_HEMI_FADE: (f32, f32) = (0.03, 0.1);

/// What a pick's texel says it met: a cluster, by its index and place, or an impostor, by its index.
const PICKED_CLUSTER: u32 = 1;
const PICKED_IMPOSTOR: u32 = 2;

/// A scene as one viewport draws it: this frame as prepared (`ViewInfo`), what the view keeps between frames
/// (`ViewState`), and what draws it (`SceneRenderer`). The scene (`LevelScene`) is shared by every view of it and passed
/// in; this prepares the view's frame from it and records it into the frame's graph.
pub struct SceneView {
  /// This frame as prepared, which its passes read.
  info: ViewInfo,
  /// What the view keeps from one frame to the next.
  state: ViewState,
  /// What draws it, and what that keeps between frames: the passes' buffers and bind groups, and the effects drawn from
  /// the view.
  renderer: SceneRenderer,
  /// Its feature units.
  features: SceneFeatures,
}

impl SceneView {
  /// What the scene's particles came to since the last report, as this view draws them.
  pub fn take_particles_report(&self, scene: &mut LevelScene) -> RenderParticlesReport {
    scene.particles.take_report(self.state.particles.drawn)
  }

  /// What it keeps from one frame to the next.
  pub fn get_state(&self) -> &ViewState {
    &self.state
  }

  pub fn get_state_mut(&mut self) -> &mut ViewState {
    &mut self.state
  }

  /// A view of `scene`, nothing of it drawn yet.
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    view_layout: &wgpu::BindGroupLayout,
    scene: &LevelScene,
  ) -> Self {
    Self {
      info: ViewInfo::default(),
      state: ViewState::new(device, queue),
      renderer: SceneRenderer::new(device, view_layout, scene.statics.cull.args.size()),
      features: SceneFeatures::new(device, queue),
    }
  }

  /// Asks for the textures its weather draws with: the rain's, the bolts', the flares' and the sky's.
  pub fn request_textures(
    &mut self,
    scene: &LevelScene,
    weather_textures: &mut WeatherTextureCache,
    (lighting, weather): (&RenderLighting, Option<&Arc<RenderLevelWeather>>),
    options: &RenderViewOptions,
  ) {
    let assets: Arc<dyn RenderAssetSource> = Arc::clone(&scene.source) as Arc<dyn RenderAssetSource>;

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
        .features
        .flares
        .request((lighting, weather), weather_textures, &assets);
      weather_textures.request_sky(&lighting.sky, options.show.is_clouded, &assets);
    } else {
      // The irradiance cubes light the hemisphere whether or not the sky is drawn.
      for reference in lighting.sky.environments.iter().flatten() {
        weather_textures.request(reference, WeatherTextureKind::Cube, &assets);
      }
    }
  }

  /// Sizes the targets to the viewport and writes what this frame's cull and lighting read.
  #[allow(clippy::too_many_arguments)]
  pub fn prepare(
    &mut self,
    scene: &mut LevelScene,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut FrameEncoder,
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
    profiling::scope!("prepare view");

    if !self.state.targets.as_ref().is_some_and(|it| it.is_sized(width, height)) {
      let targets: ViewTargets = ViewTargets::new(device, width, height);
      let pyramid: DepthPyramid = DepthPyramid::new(device, width, height);

      self.state.targets = Some(targets);
      self.state.pyramid = Some(pyramid);
      // A pyramid of another size holds no depth this frame can be tested against.
      self.state.history = None;
    }

    if let Some(overlays) = self.state.overlays.as_mut().filter(|it| it.skeleton.is_some()) {
      overlays.set_skeleton(device, &scene.skeleton_segments);
    }

    self
      .renderer
      .shadows
      .prepare(device, options.features.shadows.resolution);

    self
      .features
      .exposure
      .prepare(queue, &options.features.exposure, Instant::now());

    self.info.sun_sprite = self.features.flares.prepare(
      device,
      (lighting, weather),
      options,
      weather_textures,
      (view, weather_rate),
      self.state.targets.is_some(),
    );

    let sky = &lighting.sky;

    self.info.sky = Some(passes.sky.collect(
      weather_textures,
      (sky, self.info.sun_sprite.as_ref().map(|(texture, _)| texture.as_str())),
      &scene.environments.1,
    ));
    // The surroundings cube at the size the settings ask, made again where that changed.
    let probe: RenderReflectionProbe = options.features.reflections.probe;

    if self.state.sky_prefilter.as_ref().is_some_and(|it| it.probe != probe) {
      self.state.sky_prefilter = None;
    }

    self
      .state
      .sky_prefilter
      .get_or_insert_with(|| SkyPrefilter::new(device, probe));

    // The ambient effects blow the wind the grass, the rain and the campfires read this frame.
    let gust: AmbientGust = scene.gust;

    // What of the scene drawn by passes of its own the view may see: none of the water's or the surface passes' work
    // runs for what is out of sight.
    let sight: StaticSight = scene.statics.get_sight(&view.get_planes());

    self.features.water.prepare(
      device,
      options,
      WaterFrame {
        targets: self.state.targets.as_ref(),
        is_in_view: sight.is_water(),
        intensity: lighting.water_intensity,
        wind: lighting.wind,
        rain: lighting.rain.map_or(0.0, |rain| rain.density),
        time: scene.started.elapsed().as_secs_f32(),
      },
    );

    let sway_time: f32 = scene.started.elapsed().as_secs_f32();
    let foliage_settings: RenderFoliageSettings = options.features.grass.foliage;
    let foliage: FoliageWindValues = self.state.foliage.advance(
      sway_time,
      &foliage_settings,
      (lighting.wind, lighting.trees.is_some()),
      lighting.rain.map_or(0.0, |rain| rain.density.clamp(0.0, 1.0)),
    );
    let wind: WindUniform = WindUniform::new(lighting.trees.as_ref(), sway_time)
      .following(self.state.last_wind.as_ref())
      .with_foliage(&foliage);

    self.state.last_wind = Some(wind);
    self.state.foliage_values = foliage;

    // The enhanced motion moves a tree by more than the engine's lean for a given reach, so its shadows redraw at the
    // enhanced reach whenever it blows.
    let amplitude: f32 = if foliage_settings.is_enhanced() && foliage.wind.z > 0.0 {
      FOLIAGE_SHADOW_AMPLITUDE
    } else {
      wind.get_amplitude()
    };

    self.info.sway = (amplitude, sway_time);
    // The falling rain while the weather rains and the view shows it, then the wet surfaces, which follow it.
    let time: f32 = scene.started.elapsed().as_secs_f32();
    let windows: OverheadWindows = self.get_overhead_windows();
    let falling: Option<RainUniform> = self.features.rain.prepare(
      device,
      (lighting, weather),
      (lighting.rain.filter(|_| options.mode.is_lit), gust, time),
      (windows.cover, windows.cover_shape),
      weather_textures,
    );

    self.features.wet.prepare(
      (lighting, weather),
      options,
      (falling.as_ref(), time, weather_rate),
      (windows, self.state.noise_frame),
      weather_textures,
    );
    self
      .features
      .thunder
      .prepare(device, (lighting, weather), options, weather_textures);
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
      is_adapting: self.features.exposure.is_adapting(),
      sky_blend: match (is_first_up, is_second_up) {
        (true, false) => 0.0,
        (false, true) => 1.0,
        _ => sky.blend,
      },
      is_irradiance_up: sky
        .environments
        .iter()
        .all(|reference| reference.as_deref().is_some_and(|it| weather_textures.is_up(it))),
      clouds_time: scene.started.elapsed().as_secs_f32(),
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
    self.features.bloom.prepare(
      passes,
      options,
      lighting.engine,
      self
        .state
        .targets
        .as_ref()
        .map(|targets| (targets.width, targets.height)),
    );
    self.features.fog.prepare(
      options,
      lighting,
      self
        .state
        .targets
        .as_ref()
        .map(|targets| (targets.width, targets.height)),
      scene.started.elapsed().as_secs_f32(),
    );
    self.info.debanding = PresentUniform::get_debanding(options);

    if !options.features.is_occlusion_culled {
      self.state.history = None;
    }

    // The engine's screen: the viewport's pixels, widened for a lens narrower than its 90 degrees.
    let screen: f32 =
      (width * height) as f32 * (90.0 / field_of_view.max(1.0)).powi(2) * (EPS_S + options.features.lod.geometry_lod);
    let threshold = |area: f32| -> f32 { (area / 3.0).powi(2) / screen };

    self.info.cull = StaticCullParams {
      cluster_count: scene.statics.geometry.get_cluster_count(),
      row_count: scene.statics.geometry.get_row_count(),
      batch_count: StaticBatch::COUNT as u32,
      impostor_count: scene.statics.geometry.get_impostor_count(),
      glod_start: threshold(options.features.lod.ssa_glod_start),
      glod_end: threshold(options.features.lod.ssa_glod_end),
      discard_below: options.features.lod.ssa_discard.powi(2) / screen,
      candidate_capacity: scene.statics.cull.get_list_capacity(),
      is_occluding: options.features.is_occlusion_culled as u32,
      lod_a: threshold(options.features.lod.ssa_a),
      lod_b: threshold(options.features.lod.ssa_b),
      is_impostors: options.features.lod.is_impostors as u32,
      pad: UVec4::ZERO,
      lod_origin: view.position.extend(1.0),
    };
    self.info.matrices = (view.view, view.projection);
    self.state.grass.prepare(
      (device, &mut *encoder),
      (passes.grass, scene.grass.get_level()),
      &options.features.grass,
      view,
      self.info.cull.discard_below,
      (
        scene.started.elapsed().as_secs_f32(),
        lighting.trees.is_some(),
        gust.strength,
      ),
      &self.state.foliage_values,
      // Where the puddles stood last frame, which no tuft is planted in: the wet readies after the grass.
      (
        &self.renderer.level_surface.puddles,
        GrassPuddlesUniform::new(
          self.renderer.level_surface.get_window(),
          self.features.wet.has_puddles(),
        ),
      ),
    );
    scene.lights.prepare(
      &mut *encoder,
      &mut self.state.lights,
      LightsFrame {
        camera: view,
        settings: &options.features.lights,
        engine: lighting.engine,
        lod: (self.info.cull.glod_start, self.info.cull.glod_end),
        statics: &scene.statics,
        sway: &to_sway(&scene.statics, self.info.sway),
        campfire_shares: &scene.campfire_shares,
      },
    );
    self.info.camera = *view;
    self.info.sun_direction = lighting.get_sun_direction();
    self.info.switches = Vec4::new(
      1.0,
      options.mode.is_bumped as u32 as f32,
      options.features.hemi_strength,
      0.0,
    );
    self.info.shadow_settings = options.features.shadows.clone();
    self.info.output = output;
    self.info.debug_view = options.mode.debug_view;
    self.info.is_wireframe = options.mode.is_wireframe;
    self.info.corrections = options.features.corrections;
    self.features.resolve.prepare(
      device,
      view,
      (
        self
          .state
          .targets
          .as_ref()
          .map(|targets| (targets.width, targets.height)),
        output,
      ),
      (options.output.upscaling, options.is_scene_linear()),
    );
    self.features.smoothing.prepare(
      passes,
      options.features.antialiasing,
      (
        self.state.targets.is_some() && self.info.debug_view == RenderDebugView::Final,
        options.is_scene_linear(),
      ),
    );
    self.info.lights_settings = options.features.lights;

    self
      .features
      .ambient_occlusion
      .prepare(options, view.projection, (width.div_ceil(2), height.div_ceil(2)));
    self.features.bitmask.prepare(
      device,
      options,
      view.projection,
      (width.div_ceil(2), height.div_ceil(2)),
    );
    self.features.reflections.prepare(device, options, (width, height));
    self
      .features
      .composited
      .prepare((scene, device, &mut *encoder), view, options, &sight);

    if let Some(pyramid) = &self.state.pyramid {
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
    // The enhanced fog's height fog fades out as the camera goes indoors, by its hemi as the engine tells indoors.
    self.info.lighting.height_fog.w *= scene.camera_hemi.map_or(1.0, |hemi| {
      let share: f32 = ((hemi - INDOOR_HEMI_FADE.0) / (INDOOR_HEMI_FADE.1 - INDOOR_HEMI_FADE.0)).clamp(0.0, 1.0);

      share * share * (3.0 - 2.0 * share)
    });

    if self.features.resolve.is_temporal() {
      self.state.noise_frame = self.state.noise_frame.wrapping_add(1);
    }

    self
      .features
      .contact_shadows
      .prepare(options, self.info.lighting.to_sun, (height, self.state.noise_frame));

    scene.particles.fill(view, options, &mut self.state.particles);

    self.state.particles.upload((device, &mut *encoder));

    self.write_present(options, scene.started.elapsed().as_secs_f32());
    self.features.volumetric.prepare(device, options);
    self.prepare_shadows(scene, device, encoder, (view_layout, textures));
    self.follow_overhead_maps();
  }

  /// Writes where the rain cover and the level's surface now stand into what reads them, once `prepare_shadows` has
  /// moved them: written before, a frame redrawing one would read it through where it stood the frame before.
  fn follow_overhead_maps(&mut self) {
    let windows: OverheadWindows = self.get_overhead_windows();

    self.features.rain.follow_cover(windows.cover, windows.cover_shape);
    self.features.wet.follow(windows);
  }

  /// Where the rain cover and the level's surface stand now.
  fn get_overhead_windows(&self) -> OverheadWindows {
    let (surface, surface_shape): (Vec4, Vec4) = self.renderer.level_surface.get_window();

    OverheadWindows {
      cover: self.renderer.rain_cover.get_window(),
      cover_shape: self.renderer.rain_cover.get_shape(),
      surface,
      surface_shape,
      water: self.renderer.level_surface.get_water_window(),
    }
  }

  /// Readies this frame's shadows, sun, rain cover and lights, which the frame's passes then draw; `encoder` takes
  /// the copies a list growing makes.
  fn prepare_shadows(
    &mut self,
    scene: &LevelScene,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    (view_layout, textures): (&wgpu::BindGroupLayout, &TextureCache),
  ) {
    let SceneView {
      info,
      renderer,
      features,
      ..
    } = self;
    let SceneRenderer {
      shadows,
      rain_cover,
      level_surface,
      surroundings,
      ..
    } = renderer;
    let statics: &StaticScene = &scene.statics;
    let frame: ShadowFrame<'_> = ShadowFrame {
      scene: statics,
      camera: &info.camera,
      settings: &info.shadow_settings,
      sun_direction: info.sun_direction,
      switches: info.switches,
      sway: to_sway(statics, info.sway),
      params: &info.cull,
      textures,
    };

    shadows.prepare_cascades(device, encoder, view_layout, &frame);

    // What may wait a few frames draws on one the cascades leave light, one job a frame.
    let mut load: FrameLoad = shadows.get_load();

    // The cube is for the traced reflections alone; the engine's reflect its own.
    if features.reflections.get_settings().is_drawn() {
      surroundings.prepare(
        device,
        encoder,
        &frame,
        (features.reflections.get_settings().probe, &mut load),
      );
    } else {
      surroundings.rest();
    }

    // Out to the wet reach where the wetting reaches past the engine's.
    if features.rain.is_raining() || features.wet.is_wet() {
      let reach: Option<f32> = features.wet.get_reach();

      rain_cover.prepare(device, encoder, &frame, (reach, &mut load));
    } else {
      rain_cover.rest();
    }

    // The puddles seat on it.
    if features.wet.is_enhanced() {
      level_surface.prepare(device, encoder, &frame, (scene.lights.get_fires(), &mut load));
    }
  }

  /// Readies the scene's light shadow faces every view of it asked for this frame, from this view's culling, once all
  /// were prepared; `encoder` takes the copies a list growing makes.
  pub fn prepare_scene_shadows(
    &self,
    scene: &mut LevelScene,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    textures: &TextureCache,
  ) {
    let LevelScene { statics, lights, .. } = scene;
    let frame: ShadowFrame<'_> = ShadowFrame {
      scene: statics,
      camera: &self.info.camera,
      settings: &self.info.shadow_settings,
      sun_direction: self.info.sun_direction,
      switches: self.info.switches,
      sway: to_sway(statics, self.info.sway),
      params: &self.info.cull,
      textures,
    };

    lights.finish_shadows(device, encoder, &frame);
  }

  /// Declares the scene's culls and G-buffer draws: what last frame's depth does not hide, culled and drawn; then, while
  /// it culls occlusion, this frame's depth reduced and what the first draw does not hide of the rest culled and drawn.
  fn add_gbuffer_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    (view, textures, scene): (&'a ViewBinding, &'a wgpu::BindGroup, &StaticSceneHandles),
    (targets, pyramid): (ViewTargetHandles, &'a DepthPyramid),
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

    let (late, late_dispatch): (GraphBuffer, GraphBuffer) = (scene.late, scene.late_dispatch);
    // Each level as its own resource, the depth reduced into the first and each into the next.
    let level_views: Vec<GraphTexture> = pyramid
      .level_views
      .iter()
      .map(|view| bindings.import_view(&mut *graph, "depth pyramid level", view))
      .collect();
    let first: PyramidDepthParameters = PyramidDepthParameters {
      source_depth: depth,
      target_level: level_views[0],
    };
    let levels: Vec<PyramidLevelParameters> = level_views
      .windows(2)
      .map(|pair| PyramidLevelParameters {
        source_level: pair[0],
        target_level: pair[1],
      })
      .collect();

    levels
      .iter()
      .fold(
        graph.add_compute_pass("depth pyramid").parameters(&first),
        |builder, level| builder.parameters(level),
      )
      .record(move |context| passes.pyramid.record(context, pyramid, (&first, &levels), false));
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

  /// Declares the lighting: the lights binned into the view's clusters, the contact shadows marched, the sun then every
  /// light drawn into the light target, the binning's overflow read back, and the ambient occlusion and the indirect
  /// light searched and filtered. Answers the indirect light filtered, none where it is not gathered.
  fn add_lighting_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    (view, textures, scene): (&'a ViewBinding, &'a wgpu::BindGroup, &'a LevelScene),
    (targets, lit): (ViewTargetHandles, LightingHandles<'a>),
    has_lights: bool,
  ) -> Option<GraphTexture> {
    let lights: &'a LightsView = &self.state.lights;
    let counts: GraphBuffer = bindings.import_buffer(graph, "light cluster counts", &lights.counts);
    let records: GraphBuffer = bindings.import_buffer(graph, "light records", &lights.record_buffer);
    let items: GraphBuffer = bindings.import_buffer(graph, "light cluster items", &lights.items);
    let lights_uniform: UniformBinding<LightsUniform> = runtime.push_uniform(&lights.uniform);

    graph
      .add_encoder_pass("light counts clear")
      .buffer(counts, GraphBufferAccess::CopyDestination)
      .record(move |context| lights.clear_overflow(context.get_encoder()));

    if has_lights {
      let parameters: LightBinningParameters = LightBinningParameters {
        records: StorageArray::new(records),
        counts: StorageArrayMut::new(counts),
        items: StorageArrayMut::new(items),
        lights: lights_uniform,
      };

      graph
        .add_compute_pass("light binning")
        .parameters(&parameters)
        .record(move |context| passes.lights.record_binning(context, &parameters));
    }

    // The contact shadows the sun multiplies its own by, and the uniform the lights march theirs by.
    let (contact_shadows, contact): (GraphTexture, UniformBinding<ContactShadowUniform>) = self
      .features
      .contact_shadows
      .add_passes((&mut *graph, &mut *bindings, &mut *runtime), passes, view, targets);
    let sun: SunParameters = SunParameters {
      normal_target: targets.normal,
      material_target: targets.material,
      depth_target: targets.depth,
      material_lut: lit.material_lut,
      lut_sampler: lit.lut_sampler,
      lighting: lit.lighting,
      shadow_maps: lit.shadow_maps,
      shadows: lit.shadows,
      contact_shadows,
    };

    graph
      .add_raster_pass("sun")
      .parameters(&sun)
      .color(GraphColorAttachment::new(
        targets.light,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .record(move |context| passes.sun.record(context, view, &sun));

    if has_lights {
      let parameters: LightsParameters = LightsParameters {
        normal_target: targets.normal,
        material_target: targets.material,
        depth_target: targets.depth,
        material_lut: lit.material_lut,
        lut_sampler: lit.lut_sampler,
        records: StorageArray::new(records),
        counts: StorageArray::new(counts),
        items: StorageArray::new(items),
        lights: lights_uniform,
        shadow_atlas: bindings.import_view(graph, "light shadow atlas", scene.lights.get_shadow_atlas()),
        contact,
      };
      let is_contact: bool = self.features.contact_shadows.is_lit();

      graph
        .add_raster_pass("lights")
        .parameters(&parameters)
        .color(GraphColorAttachment::new(targets.light, wgpu::LoadOp::Load))
        .record(move |context| {
          passes
            .lights
            .record_draw(context, view, &parameters, (textures, is_contact))
        });
    }

    // The overflow, read back for a report a frame or more later: an effect the graph cannot see.
    graph
      .add_encoder_pass("light overflow")
      .buffer(counts, GraphBufferAccess::CopySource)
      .keep()
      .record(move |context| lights.record_overflow(context.get_encoder()));

    // GTAO searches on its own; VBAO, the indirect light or both share one bitmask search.
    self
      .features
      .ambient_occlusion
      .add_passes((&mut *graph, &mut *runtime), passes, view, targets);

    self
      .features
      .bitmask
      .add_passes((graph, bindings, runtime), passes, view, targets)
  }

  /// Adds the passes filling the prefiltered sky cube this frame, a mip each: the first the surroundings cube's face
  /// drawn this frame, the level around the camera lit with the sky behind, or the sky's cubes alone as the sky is
  /// drawn where none is; each after it from the mips before it over its roughness's lobe.
  #[allow(clippy::too_many_arguments)]
  fn add_sky_prefilter_passes<'a>(
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    sky: &SkyParameters<'a>,
    prefilter: &'a SkyPrefilter,
    lit: &LightingHandles<'a>,
    surroundings: Option<(
      &'a SurroundingsCube,
      &StaticSceneHandles,
      (&'a StaticCullParams, &'a wgpu::BindGroup),
    )>,
  ) {
    let (lighting, exposure): (UniformBinding<LightingUniform>, GraphBuffer) = (lit.lighting, lit.exposure);
    let empty: GraphTexture = bindings.import_view(&mut *graph, "sky prefilter none", passes.sky_prefilter.get_empty());

    for (index, level) in prefilter.levels.iter().enumerate() {
      let source: GraphTexture = level.source.as_ref().map_or(empty, |before| {
        bindings.import_view(&mut *graph, "sky prefilter before", before)
      });
      let faces: GraphTexture = bindings.import_view(&mut *graph, "sky prefilter level", &level.target);

      if let (0, Some((cube, scene, cull))) = (index, surroundings) {
        cube.add_passes(
          (&mut *graph, &mut *bindings, runtime),
          passes,
          scene,
          cull,
          (lit, sky),
          faces,
        );

        continue;
      }
      let settings: UniformBinding<SkyPrefilterUniform> = runtime.push_uniform(&SkyPrefilterUniform::new(
        index as u32,
        prefilter.levels.len() as u32,
        (level.size, prefilter.probe.get_size()),
        SkyPrefilterPass::SAMPLES,
      ));
      let parameters: SkyPrefilterParameters = SkyPrefilterParameters {
        sky_cube_0: sky.sky_cube_0,
        sky_cube_1: sky.sky_cube_1,
        sky_clouds_0: sky.sky_clouds_0,
        sky_clouds_1: sky.sky_clouds_1,
        sky_clamp: sky.sky_clamp,
        sky_repeat: sky.sky_repeat,
        source,
        lighting,
        exposure: StorageValue::new(exposure),
        settings,
        faces,
      };
      let size: u32 = level.size;

      graph
        .add_compute_pass(SkyPrefilterPass::PASSES[index])
        .parameters(&parameters)
        .record(move |context| passes.sky_prefilter.record(context, size, &parameters));
    }
  }

  /// Ends a frame its graph recorded: the water's reflection, the bitmask search's accumulations and the temporal
  /// resolve's history it wrote become the ones the next frame keeps.
  pub fn end_frame(&mut self, frame: &LevelFrame) {
    if frame.is_lit {
      self.features.water.finish_frame();

      self.features.bitmask.end_frame();

      if let Some(targets) = &mut self.state.targets {
        self.features.reflections.end_frame(&mut targets.depth);
      }

      self.features.volumetric.end_frame();
    }

    self.features.resolve.end_frame();
  }

  /// Notes what the present pass reads, once the frame knows what draws into the distortion target: `def_distort`
  /// while the water, a particle or a distorting surface of the scene does, nothing otherwise; which bloom it lays
  /// over the frame, and the sky debanding, its noise turning with `time`.
  fn write_present(&mut self, options: &RenderViewOptions, time: f32) {
    let is_glass_distorting: bool = self.features.composited.is_distorting();

    let Some(targets) = &self.state.targets else {
      return;
    };
    let water: &RenderWaterSettings = &options.features.water;
    let is_water_distorting: bool = water.is_enabled && water.is_distorted && options.mode.is_lit;
    let is_distorting: bool = !options.mode.is_wireframe
      && (is_water_distorting || self.state.particles.is_distorting() || is_glass_distorting);

    let present: PresentUniform = PresentUniform::new(
      self.info.debug_view,
      (
        self.features.ambient_occlusion.is_drawn(),
        self.features.bitmask.get_search().is_some_and(|search| search.is_lit),
        self.features.reflections.get_trace().map(ReflectionTrace::get_ratio),
      ),
      !targets.is_sized(self.info.output.width, self.info.output.height),
      if is_distorting { water.distortion } else { 0.0 },
      self.info.output,
      &self.info.corrections,
      (self.info.selection_color, self.features.bloom.is_engine_drawn()),
    );
    let present: PresentUniform = if self.features.bloom.is_enhanced_drawn() {
      present.with_enhanced_bloom()
    } else {
      present
    }
    .with_linear_scene(options.is_scene_linear());

    self.info.present = match self.info.debanding {
      Some(quality) => present.with_debanding(quality.get_passes(), options.features.debanding.radius, time),
      None => present,
    };
  }

  /// Readies a frame for its graph, none before the view's targets are made: a pick's camera narrowed to the texel
  /// picked and the readback it is copied into while one is free, and the depth the next frame's occlusion tests.
  pub fn begin_frame(
    &mut self,
    (device, encoder): (&wgpu::Device, &mut FrameEncoder),
    pick: Option<(&CameraUniform, &wgpu::BindGroupLayout, bool)>,
  ) -> Option<LevelFrame> {
    if self.state.targets.is_none() || self.state.pyramid.is_none() {
      return None;
    }

    let pick_slot: Option<usize> =
      pick.and_then(|(camera, view_layout, _)| self.ready_pick(device, encoder, camera, view_layout));

    if self.info.cull.is_occluding != 0 {
      self.state.history = Some(self.info.matrices);
    }

    Some(LevelFrame {
      pick_slot,
      is_pick_through: pick.is_some_and(|(_, _, is_through)| is_through),
      is_scene_first: false,
      is_lit: self.state.targets.is_some(),
    })
  }

  /// Declares the frame `begin_frame` readied into the frame's graph: culls the scene and draws it into the G-buffer
  /// (what last frame's depth does not hide, then, culling occlusion, what this frame's first draw does not hide of the
  /// rest, leaving this frame's depth reduced for the next), shadows and lights it, draws the water and what blends over
  /// it, resolves the frame, and draws the pick last. Which passes run is decided here; each reads the view, which
  /// recording leaves as it is. The present pass reads what it leaves.
  pub fn record<'a>(
    &'a self,
    scene: &'a LevelScene,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    textures: &'a TextureCache,
    frame: &LevelFrame,
  ) -> Option<SceneOutput> {
    profiling::scope!("declare");

    let is_occluding: bool = self.info.cull.is_occluding != 0;
    let pick_slot: Option<usize> = frame.pick_slot;
    let is_pick_through: bool = frame.is_pick_through;
    let is_drawn: bool = !self.info.is_wireframe;
    let is_wallmarked: bool = self.info.is_wallmarked && is_drawn;
    let is_raining: bool = self.features.rain.is_raining();
    let is_wet: bool = self.features.wet.is_wet();
    let is_lit: bool = frame.is_lit;
    let has_lights: bool = self.state.lights.get_count() > 0;
    // The shadowed lights' shafts, drawn into a target of their own for the sun shafts' composite to add.
    let is_froxeled: bool = is_lit && self.features.volumetric.is_froxeled();
    let light_atlas: &'a wgpu::TextureView = scene.lights.get_shadow_atlas();
    let has_sky: bool = self.info.sky.is_some();
    let is_hazing: bool = has_sky && self.info.is_hazing;
    let is_composited: bool = is_drawn && has_sky;
    let has_particles: bool = is_drawn && self.state.particles.is_drawing();
    let is_shafted: bool = self.info.is_shafted && !is_froxeled;
    let particle_surfaces: &wgpu::Buffer = scene.particles.get_surfaces();
    let texture_group: &wgpu::BindGroup = textures.get_bind_group();
    let scene_view: &'a SceneView = self;
    let (Some(targets), Some(pyramid)) = (&scene_view.state.targets, &scene_view.state.pyramid) else {
      return None;
    };
    let unlit: GraphTexture = bindings.import_view(&mut *graph, "indirect light none", passes.vbao.get_dark());
    let untraced: GraphTexture =
      bindings.import_view(&mut *graph, "reflections none", passes.reflections.get_untraced());
    let no_ends: GraphTexture =
      bindings.import_view(&mut *graph, "reflection ends none", passes.reflections.get_no_ends());
    let handles: ViewTargetHandles = ViewTargetHandles::import(&mut *graph, &mut *bindings, targets);
    let statics: StaticSceneHandles = StaticSceneHandles::import(
      (&mut *graph, &mut *bindings, runtime),
      &scene.statics,
      &pyramid.view,
      (&scene_view.info.cull, &scene_view.info.occlusion, &scene_view.info.wind),
    );
    let layouts: [StaticDrawParameters; StaticLayout::COUNT] = statics.get_camera_draws().layouts;
    let lit: LightingHandles = LightingHandles {
      lighting: runtime.push_uniform(&scene_view.info.lighting),
      exposure: bindings.import_buffer(&mut *graph, "exposure", &scene_view.features.exposure.state),
      material_lut: bindings.import_view(&mut *graph, "material table", &passes.table.view),
      lut_sampler: &passes.table.sampler,
      shadow_maps: bindings.import_view(
        &mut *graph,
        "sun shadow maps",
        &scene_view.renderer.shadows.get_maps().view,
      ),
      shadows: runtime.push_uniform(scene_view.renderer.shadows.get_values()),
    };

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

    let grass_level: Option<&GrassLevel> = scene.grass.get_level();
    let grass_args: Option<GraphBuffer> =
      scene_view
        .state
        .grass
        .add_planting(&mut *graph, &mut *bindings, (passes.grass, grass_level));

    scene_view.add_gbuffer_passes(
      (&mut *graph, &mut *bindings),
      passes,
      (view, texture_group, &statics),
      (handles, pyramid),
      is_occluding,
    );

    // The depth the reflections walk their rays over, kept before the grass draws, or after it where the grass is
    // reflected.
    let is_grass_reflected: bool = scene_view.features.reflections.get_settings().is_grass_reflected;

    if !is_grass_reflected {
      scene_view
        .features
        .reflections
        .add_pyramid_passes((&mut *graph, &mut *bindings), passes, handles.depth);
    }

    let stats_args: GraphBuffer = statics.args;

    // The cull's counts, read back for a report a frame or more later: an effect the graph cannot see.
    graph
      .add_encoder_pass("stats")
      .buffer(stats_args, GraphBufferAccess::CopySource)
      .keep()
      .record(move |context| {
        let args: &wgpu::Buffer = context.get_buffer(stats_args);

        scene_view
          .state
          .stats
          .record(context.get_encoder(), args, StaticCullScratch::STATS_OFFSET);
      });

    if let (true, Some(grass_args)) = (is_drawn, grass_args) {
      scene_view.state.grass.add_draw(
        &mut *graph,
        (passes.grass, grass_level),
        (handles, grass_args),
        (view, texture_group),
      );
    }

    if is_grass_reflected {
      scene_view
        .features
        .reflections
        .add_pyramid_passes((&mut *graph, &mut *bindings), passes, handles.depth);
    }

    if is_wallmarked {
      let args: Vec<GraphBuffer> = Self::list_draw_args(&statics, &scene_view.info.cull);

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

    scene_view.renderer.shadows.add_passes(
      (&mut *graph, &mut *bindings),
      passes,
      (&statics, lit.shadow_maps),
      (&scene_view.info.cull, texture_group),
    );

    // The rain's cover, which its passes draw and reduce and the rain and the wet surfaces read.
    let covers: Option<CoverTextures> = (is_raining || is_wet).then(|| {
      scene_view.renderer.rain_cover.add_passes(
        (&mut *graph, &mut *bindings),
        passes,
        &statics,
        (&scene_view.info.cull, texture_group),
      )
    });
    let cover: Option<GraphTexture> = covers.map(|covers| covers.depth);
    let wet: Option<UniformBinding<WetUniform>> = scene_view.features.wet.push_uniform(runtime);

    // The level's surface seen from overhead, its lowest heights, its puddle sites and its water, which the enhanced
    // wetting places its puddles by.
    let [surface_lowest, puddles, surface_water]: [GraphTexture; 3] = match wet {
      Some(wet) if scene_view.features.wet.is_enhanced() => {
        let water: UniformBinding<WaterUniform> = runtime.push_uniform(scene_view.features.water.get_uniform());
        let keep: PuddleKeepInputs = PuddleKeepInputs {
          wet,
          noise: &passes.water.get_maps().perlin,
          sampler: passes.wet.get_sampler(),
        };

        scene_view.renderer.level_surface.add_passes(
          (&mut *graph, &mut *bindings, runtime),
          passes,
          &statics,
          (&scene_view.info.cull, texture_group),
          (water, keep),
        )
      }
      _ => {
        let surface: &LevelSurface = &scene_view.renderer.level_surface;

        [
          bindings.import_view(&mut *graph, "level surface lowest", &surface.lowest),
          bindings.import_view(&mut *graph, "puddles", &surface.puddles),
          bindings.import_view(&mut *graph, "level water", &surface.water.depth),
        ]
      }
    };

    if frame.is_scene_first {
      scene.lights.add_shadow_passes(
        (&mut *graph, &mut *bindings),
        passes,
        &statics,
        (&scene_view.info.cull, texture_group),
      );
    }

    graph.begin_group("lighting");

    // The rain wets the G-buffer before any light is drawn over it: the wet surface the enhanced patch writes, which the
    // reflections and combine read, and the gloss the reflections' wet reach adds, which combine's lights leave out;
    // a texel of nothing for each while dry.
    let (wet_surface, wet_reach): (GraphTexture, GraphTexture) = scene_view
      .features
      .wet
      .add_passes(
        (&mut *graph, &mut *bindings),
        passes,
        view,
        (handles, covers, wet),
        [surface_lowest, puddles, surface_water],
      )
      .unwrap_or_else(|| {
        let none: &'a wgpu::TextureView = passes.reflections.get_empty();

        (
          bindings.import_view(&mut *graph, "wet surface none", none),
          bindings.import_view(&mut *graph, "wet reach none", none),
        )
      });

    // The indirect light combine adds and a debug view shows, or a texel of none; the reflections traced, and how
    // their rays ended.
    let mut indirect_light: Option<GraphTexture> = None;
    let mut reflections: Option<GraphTexture> = None;
    let mut reflection_ends: Option<GraphTexture> = None;
    let mut enhanced_bloom: Option<GraphTexture> = None;

    // The finished frame at the drawn size, and the image the window shows where that differs: each stage of the chain
    // writes a target of its own that the next reads, so nothing is copied back.
    let mut frame: GraphTexture = handles.scene;
    let mut shown: Option<GraphTexture> = None;

    if is_lit {
      indirect_light = scene_view.add_lighting_passes(
        (&mut *graph, &mut *bindings, runtime),
        passes,
        (view, texture_group, scene),
        (handles, lit),
        has_lights,
      );

      let sky: Option<SkyParameters> = match (&scene_view.info.sky, &scene_view.state.sky_prefilter) {
        (Some(views), Some(prefilter)) => {
          let sky: SkyParameters = passes.sky.import((views, &prefilter.cube), &mut *graph, &mut *bindings);

          let surroundings: &SurroundingsCube = &scene_view.renderer.surroundings;

          Self::add_sky_prefilter_passes(
            (&mut *graph, &mut *bindings, runtime),
            passes,
            &sky,
            prefilter,
            &lit,
            surroundings
              .is_active()
              .then_some((surroundings, &statics, (&scene_view.info.cull, texture_group))),
          );

          Some(sky)
        }
        _ => None,
      };
      // The sky's group, which the water's scene pass reads too once the water's own sky views shadow the name.
      let sky_parameters: Option<SkyParameters> = sky;

      if let (true, Some(sky)) = (is_hazing, sky) {
        let parameters: SkyHazeParameters = SkyHazeParameters {
          lighting: lit.lighting,
          exposure: StorageValue::new(lit.exposure),
        };

        graph
          .add_raster_pass("haze")
          .parameters(&parameters)
          .parameters(&sky)
          .color(GraphColorAttachment::new(
            handles.haze,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .record(move |context| passes.sky_haze.record(context, &parameters, &sky));
      }

      // The reflections and combine record in a group of their own, encoded beside the lights before them.
      graph.begin_group("reflections");

      // The last frame's scene and depth the reflections keep: read by this frame's hits, written again after the
      // particles draw.
      let source: Option<(GraphTexture, GraphTexture)> = scene_view
        .features
        .reflections
        .import_source((&mut *graph, &mut *bindings));

      let traced: Option<(TraceParameters, &wgpu::BindGroup)> = scene_view.add_scene_trace(
        scene,
        (&mut *graph, &mut *bindings, runtime),
        passes,
        (&statics, grass_level, texture_group),
      );

      if let Some(sky) = sky {
        if let Some((history, ends)) = scene_view.features.reflections.add_passes(
          (&mut *graph, &mut *bindings, runtime),
          passes,
          view,
          (handles, lit, wet_surface),
          source,
          (&sky, traced),
        ) {
          reflections = Some(history);
          reflection_ends = Some(ends);
        }

        let parameters: CombineParameters = CombineParameters {
          albedo_target: handles.albedo,
          normal_target: handles.normal,
          material_target: handles.material,
          depth_target: handles.depth,
          light_target: handles.light,
          material_lut: lit.material_lut,
          lut_sampler: lit.lut_sampler,
          lighting: lit.lighting,
          exposure: StorageValue::new(lit.exposure),
          occlusion_target: handles.occlusion[0],
          haze_map: handles.haze,
          indirect_light: indirect_light.unwrap_or(unlit),
          reflections: reflections.unwrap_or(untraced),
          wet_surface,
          wet_reach,
        };

        graph
          .add_raster_pass("combine")
          .parameters(&parameters)
          .parameters(&sky)
          .color(GraphColorAttachment::new(
            handles.scene,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .color(GraphColorAttachment::new(
            handles.high,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .record(move |context| passes.combine.record(context, view, &parameters, &sky));
      }

      let scene: GraphTexture = handles.scene;

      // The temporal resolve's reactive weight is what the water, the composited surfaces and the particles change of
      // the frame drawn so far.
      let opaque: Option<GraphTexture> = scene_view
        .features
        .resolve
        .add_reactive_passes((&mut *graph, &mut *bindings), (scene, handles.size));

      if let (true, Some(sky)) = (scene_view.features.water.is_drawn(), scene_view.info.sky.as_ref()) {
        let draw: WaterDraw<'_> = WaterDraw {
          targets: handles,
          view,
          textures: texture_group,
          layouts,
          args: Self::list_draw_args(&statics, &scene_view.info.cull),
          lit,
          skies: [&sky.cubes[0], &sky.cubes[1]],
          sky_sampler: passes.sky.get_clamp(),
          water: &scene_view.features.water,
          scene_trace: passes.scene_trace,
          traced: sky_parameters
            .zip(traced)
            .map(|(sky, (trace, textures))| (sky, trace, textures)),
        };

        passes.water.add_passes(&mut *graph, &mut *bindings, runtime, draw);
      }

      graph.begin_group("post");

      let composited_draw: Option<(CompositedDraw, SkyParameters)> = sky.filter(|_| is_composited).map(|sky| {
        (
          CompositedDraw {
            view,
            textures: texture_group,
            handles,
            statics,
            layouts,
            args: Self::list_draw_args(&statics, &scene_view.info.cull),
            composited: CompositedParameters {
              lighting: lit.lighting,
              exposure: StorageValue::new(lit.exposure),
              material_lut: lit.material_lut,
              lut_sampler: lit.lut_sampler,
              shadow_maps: lit.shadow_maps,
              shadows: lit.shadows,
            },
          },
          sky,
        )
      });

      if let Some((draw, sky)) = &composited_draw {
        scene_view
          .features
          .composited
          .add_passes(&mut *graph, passes, (draw, *sky));
      }

      if has_particles {
        scene_view.state.particles.add_passes(
          (&mut *graph, &mut *bindings),
          passes.particles,
          (handles, lit.lighting, particle_surfaces),
          (view, texture_group),
        );
      }

      if let Some((draw, sky)) = &composited_draw {
        scene_view.features.composited.add_after_particles(
          (&mut *graph, &mut *bindings, runtime),
          passes,
          (draw, *sky, lit),
        );
      }

      // What a reflection reads a frame later: the scene with the water and the particles drawn, before the sun shafts,
      // the rain and the flares, which belong to this camera and not to what it reflects. The depth it stood on is the
      // target itself, which the reflections keep at the frame's end.
      if let Some((scene, _)) = source {
        add_texture_copy(&mut *graph, "reflection source scene", handles.scene, scene);
      }

      // The shadowed lights' shafts (`accum_volumetric`), or the enhanced volume's froxels in place of both shafts, for
      // the composite below.
      let light_volumes: Option<GraphTexture> = scene_view.features.volumetric.add_light_volume_passes(
        (&mut *graph, &mut *bindings, runtime),
        passes,
        (view, texture_group),
        (handles, &scene_view.state.lights, light_atlas),
      );
      let froxels: Option<(UniformBinding<FroxelUniform>, GraphTexture)> =
        scene_view.features.volumetric.add_froxel_passes(
          (&mut *graph, &mut *bindings, runtime),
          passes,
          view,
          (lit, light_atlas, &scene_view.state.lights),
        );

      if is_shafted || light_volumes.is_some() || froxels.is_some() {
        let (width, height) = handles.size;
        let empty: GraphTexture = bindings.import_view(&mut *graph, "sun shafts none", passes.reflections.get_empty());
        let no_froxels: GraphTexture = bindings.import_view(&mut *graph, "froxels none", passes.froxels.get_empty());
        let (froxel_uniform, froxel_integrated) =
          froxels.unwrap_or_else(|| (runtime.push_uniform(&FroxelUniform::default()), no_froxels));
        let march: SunShaftsParameters = SunShaftsParameters {
          depth_target: handles.depth,
          shadow_maps: lit.shadow_maps,
          shadows: lit.shadows,
          lighting: lit.lighting,
          exposure: StorageValue::new(lit.exposure),
          shafts: empty,
          light_volumes: empty,
          froxels: froxel_uniform,
          froxel_integrated: no_froxels,
          froxel_sampler: passes.froxels.get_sampler(),
        };
        let mut parameters: SunShaftsParameters = SunShaftsParameters {
          light_volumes: light_volumes.unwrap_or(empty),
          froxel_integrated,
          ..march
        };

        if is_shafted {
          let marched: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
            "sun shafts marched",
            width.div_ceil(2),
            height.div_ceil(2),
            SunShaftsPass::MARCHED,
          ));

          parameters.shafts = marched;
          graph
            .add_raster_pass("sun shafts march")
            .parameters(&march)
            .color(GraphColorAttachment::new(
              marched,
              wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            ))
            .record(move |context| passes.sun_shafts.record(context, view, &march, true));
        }

        graph
          .add_raster_pass("sun shafts")
          .parameters(&parameters)
          .color(GraphColorAttachment::new(handles.scene, wgpu::LoadOp::Load))
          .color(GraphColorAttachment::new(handles.high, wgpu::LoadOp::Load))
          .record(move |context| passes.sun_shafts.record(context, view, &parameters, false));
      }

      scene_view
        .features
        .rain
        .add_passes((&mut *graph, &mut *bindings, runtime), passes, view, (handles, cover));
      scene_view
        .features
        .thunder
        .add_passes((&mut *graph, &mut *bindings, runtime), passes, view, handles);

      scene_view.features.flares.add_passes(
        (&mut *graph, &mut *bindings, runtime),
        passes.flares,
        (handles, lit),
        view,
      );

      scene_view
        .features
        .fog
        .add_passes((&mut *graph, &mut *bindings, runtime), passes, view, (handles, lit));

      enhanced_bloom = scene_view.features.bloom.add_enhanced_passes(
        (&mut *graph, &mut *bindings, runtime),
        passes,
        view,
        (handles, (targets.width, targets.height)),
      );

      // A selected composited surface, glass, marked where the present outlines the selection, once the enhanced
      // bloom has read the marks.
      if let (Some((draw, _)), Some(_)) = (&composited_draw, scene_view.info.selection_color) {
        scene_view
          .features
          .composited
          .add_selection_passes(&mut *graph, passes, draw);
      }

      scene_view
        .features
        .bloom
        .add_engine_passes((&mut *graph, runtime), passes, handles);

      frame = scene_view.features.smoothing.add_passes(
        (&mut *graph, &mut *bindings, runtime),
        passes,
        (frame, (targets.width, targets.height)),
      );

      let resolved: (GraphTexture, GraphTexture) = scene_view.features.resolve.add_passes(
        (&mut *graph, &mut *bindings, runtime),
        passes,
        view,
        (handles, frame, opaque),
      );

      (frame, shown) = (resolved.0, Some(resolved.1));

      scene_view
        .features
        .exposure
        .add_passes((&mut *graph, runtime), passes, (handles.high, lit.exposure));
    }

    if let (Some(slot), Some(target), Some(pick_view)) =
      (pick_slot, &scene_view.state.pick_target, &scene_view.state.pick_view)
    {
      scene_view.add_pick_passes(
        (&mut *graph, &mut *bindings),
        passes.gbuffer,
        (pick_view, texture_group, &statics),
        (target, slot, is_pick_through),
      );
    }

    Some(SceneOutput {
      targets: handles,
      scene: frame,
      shown: shown.unwrap_or(frame),
      present: runtime.push_uniform(&scene_view.info.present),
      lighting: lit.lighting,
      indirect_light: indirect_light.unwrap_or(unlit),
      reflections: reflections.unwrap_or(untraced),
      reflection_ends: reflection_ends.unwrap_or(no_ends),
      bloom: enhanced_bloom.unwrap_or(handles.bloom[0]),
    })
  }

  /// The level's geometry a ray the screen does not answer is traced against, as one parameter set a frame which every
  /// consumer binds (the reflections, the water), where the device traces rays; each grass model's colour averaged
  /// first, which a traced hit on grassed ground blends towards.
  fn add_scene_trace<'a>(
    &'a self,
    scene: &'a LevelScene,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    (statics, grass_level, texture_group): (&StaticSceneHandles, Option<&'a GrassLevel>, &'a wgpu::BindGroup),
  ) -> Option<(TraceParameters<'a>, &'a wgpu::BindGroup)> {
    // Each grass model's colour, which a traced hit on grassed ground blends towards, averaged while the scene traces.
    let grass_colours: Option<GraphBuffer> = scene
      .trace
      .as_ref()
      .filter(|_| passes.scene_trace.is_traced())
      .zip(grass_level)
      .map(|(_, grass)| {
        let colours: GraphBuffer = bindings.import_buffer(&mut *graph, "grass colours", &grass.colours);
        let parameters: GrassColourParameters = GrassColourParameters {
          grass_models: StorageArray::new(bindings.import_buffer(&mut *graph, "grass models", &grass.models)),
          grass_colours: StorageArrayMut::new(colours),
        };
        let models: u32 = grass.model_count;

        graph
          .add_compute_pass("grass colours")
          .parameters(&parameters)
          .record(move |context| passes.grass_colours.record(context, &parameters, texture_group, models));

        colours
      });

    scene
      .trace
      .as_ref()
      .filter(|_| passes.scene_trace.is_traced())
      .map(|trace| {
        (
          TraceParameters {
            trace_scene: trace.get_structure(),
            trace_instances: StorageArray::new(bindings.import_buffer(
              &mut *graph,
              "static trace instances",
              trace.get_instances(),
            )),
            trace_records: StorageArray::new(bindings.import_buffer(
              &mut *graph,
              "static trace records",
              trace.get_records(),
            )),
            trace_places: StorageArray::new(statics.places),
            trace_baked_words: StorageArray::new(statics.words[StaticLayout::Baked.get_index()]),
            trace_tree_words: StorageArray::new(statics.words[StaticLayout::Tree.get_index()]),
            trace_model_words: StorageArray::new(statics.words[StaticLayout::Model.get_index()]),
            trace_indices: StorageArray::new(statics.indices),
            trace_surfaces: StorageArray::new(statics.surfaces),
            trace_lights: StorageArray::new(bindings.import_buffer(
              &mut *graph,
              "light records",
              &self.state.lights.record_buffer,
            )),
            trace_light_view: runtime.push_uniform(&self.state.lights.uniform),
            trace_shadow_atlas: bindings.import_view(
              &mut *graph,
              "light shadow atlas",
              scene.lights.get_shadow_atlas(),
            ),
            trace_grass_grid: StorageArray::new(bindings.import_buffer(
              &mut *graph,
              "grass grid",
              grass_level.map_or(trace.get_empty(), |grass| &grass.cells),
            )),
            trace_grass_slots: StorageArray::new(bindings.import_buffer(
              &mut *graph,
              "grass slots",
              grass_level.map_or(trace.get_empty(), |grass| &grass.slots),
            )),
            trace_grass_models: StorageArray::new(bindings.import_buffer(
              &mut *graph,
              "grass models",
              grass_level.map_or(trace.get_empty(), |grass| &grass.models),
            )),
            trace_grass_colours: StorageArray::new(
              grass_colours.unwrap_or_else(|| bindings.import_buffer(&mut *graph, "grass colours", trace.get_empty())),
            ),
            trace_grass: runtime.push_uniform(self.state.grass.get_trace_uniform()),
          },
          texture_group,
        )
      })
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
    encoder: &mut FrameEncoder,
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
      .write(encoder, camera);

    Some(slot)
  }

  /// Declares a pick: the frame's visible clusters drawn again into its texel, through its narrowed camera, after every
  /// cull of the frame, and the texel copied out into its readback.
  fn add_pick_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    gbuffer: &'a StaticGBufferPass,
    (view, textures, scene): (&'a ViewBinding, &'a wgpu::BindGroup, &StaticSceneHandles),
    (target, slot, is_through): (&'a PickTarget, usize, bool),
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

        gbuffer.pick(context, (view, &draws, textures), &args, is_through);
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
    scene: &LevelScene,
    [kind, cluster, place, depth]: [u32; 4],
    unproject: impl Fn(f32) -> Vec3,
  ) -> Option<RenderLevelHit> {
    let point: [f32; 3] = unproject(f32::from_bits(depth)).to_array();

    match kind {
      PICKED_CLUSTER => {}
      PICKED_IMPOSTOR => {
        return scene
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

    let (info, instance) = scene.statics.resolve_pick(cluster, place)?;

    if info.sector == StaticSlotInfo::NO_SECTOR {
      return scene
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
    self.features.resolve.next_jitter(options, ratio)
  }

  /// Asks for the counts recorded with the frame just submitted.
  pub fn request_stats(&self) {
    self.state.stats.request();
    self.state.lights.request_report();
  }

  /// Drops what a frame that was never submitted copied out for reading back: its counts and its pick.
  pub fn abandon_frame(&self) {
    self.state.stats.abandon();
    self.state.lights.abandon_report();

    if let Some(target) = &self.state.pick_target {
      target.abandon();
    }
  }

  /// What its frames are drawn with, as resolved from what `options` asked; the weather's light is the viewport's to add.
  pub fn describe_applied(&self, options: &RenderViewOptions) -> RenderAppliedReport {
    let antialiasing: RenderAntialiasing = self
      .features
      .resolve
      .get_temporal()
      .or(self.features.smoothing.get_mode())
      .unwrap_or(RenderAntialiasing::None);
    let cascades: usize = self.info.shadow_settings.get_cascade_count();

    RenderAppliedReport {
      antialiasing,
      render_scale: self.features.resolve.get_upscaling().scale,
      shadows: (cascades > 0).then(|| RenderAppliedShadows {
        cascades: self.info.shadow_settings.cascades[..cascades].to_vec(),
        resolution: self.renderer.shadows.get_maps().resolution,
        filter: self.info.shadow_settings.filter,
      }),
      ambient_occlusion: self
        .features
        .ambient_occlusion
        .is_drawn()
        .then_some(self.features.ambient_occlusion.get_quality()),
      indirect_light: self
        .features
        .bitmask
        .get_search()
        .filter(|search| search.is_lit)
        .map(|search| RenderAppliedIndirectLight {
          intensity: self.features.bitmask.get_indirect_light().intensity,
          is_shared: search.is_occluding,
        }),
      reflections: self
        .features
        .reflections
        .get_trace()
        .map(|trace| RenderAppliedReflections {
          intensity: trace.intensity,
          quality: trace.quality,
        }),
      height_fog: self.features.fog.get_applied(),
      debanding: self.info.debanding,
      bloom: if self.features.bloom.is_enhanced_drawn() {
        Some(RenderBloomMode::Enhanced)
      } else {
        self.features.bloom.is_engine_drawn().then_some(RenderBloomMode::Engine)
      },
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
        .and_then(|_| self.features.flares.get_shown())
        .map(str::to_owned),
    }
  }

  /// The static draws' pools and what the latest counted frame's cull kept and hid, and its lights.
  pub fn take_stats(&mut self, scene: &LevelScene) -> (RenderStaticReport, RenderLightsReport) {
    let [kept_clusters, kept_triangles, occluded_clusters, occluded_triangles] = self.state.stats.take();
    let pools: RenderStaticReport = scene.statics.get_pools();
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
    scene: &LevelScene,
    device: &wgpu::Device,
    overlays: &[RenderOverlay],
    version: u64,
    selection: Option<&RenderSelection>,
  ) {
    let boxed: Option<RenderOverlay> = selection.and_then(|selection| match selection.target {
      RenderSelectionTarget::Spawn { object } => scene
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
  pub fn resolve_selection(
    &mut self,
    scene: &LevelScene,
    selection: Option<&RenderSelection>,
  ) -> Option<&StaticSelection> {
    self.info.selection_color = None;

    let selection: &RenderSelection = selection?;
    let contents: usize = scene.statics.get_contents();

    if self
      .state
      .selection
      .as_ref()
      .is_none_or(|(target, at, _)| *target != selection.target || *at != contents)
    {
      self.state.selection = Some((
        selection.target,
        contents,
        scene.statics.resolve_selection(&selection.target),
      ));
    }

    let resolved: &StaticSelection = self.state.selection.as_ref()?.2.as_ref()?;

    self.info.selection_color = Some(selection.color);

    Some(resolved)
  }

  /// What it draws over its frame, once made.
  pub fn get_overlays(&self) -> Option<&LevelOverlays> {
    self.state.overlays.as_ref()
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
