use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, UVec4, Vec2, Vec3, Vec4};
use xrf_engine_target::XrayEngine;
use xrf_error::XrfResult;
use xrf_material::XraySurfaceDraw;
use xrf_math::EPS_S;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphDepthAttachment, GraphRuntime,
  GraphTexture, GraphTextureAccess, GraphTextureDescriptor, RasterPassBuilder, StorageArray, StorageArrayMut,
  StorageValue, StorageValueMut, UniformBinding,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_ambient_occlusion_quality::RenderAmbientOcclusionQuality;
use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_applied_indirect_light::RenderAppliedIndirectLight;
use crate::contract::render_applied_reflections::RenderAppliedReflections;
use crate::contract::render_applied_report::RenderAppliedReport;
use crate::contract::render_applied_shadows::RenderAppliedShadows;
use crate::contract::render_bloom_settings::RenderBloomSettings;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_foliage_settings::RenderFoliageSettings;
use crate::contract::render_level_hit::RenderLevelHit;
use crate::contract::render_lights_report::RenderLightsReport;
use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_particles_report::RenderParticlesReport;
use crate::contract::render_pool_use::RenderPoolUse;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_selection::RenderSelection;
use crate::contract::render_selection_target::RenderSelectionTarget;
use crate::contract::render_shadow_settings::RenderShadowSettings;
use crate::contract::render_static_report::RenderStaticReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_water_settings::RenderWaterSettings;
use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::fsr_targets::FsrTargets;
use crate::frame::indirect_light_history::IndirectLightHistory;
use crate::frame::pick_target::PickTarget;
use crate::frame::reflection_depth::ReflectionDepth;
use crate::frame::reflection_history::ReflectionHistory;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::temporal_history::TemporalHistory;
use crate::frame::temporal_jitter::TemporalJitter;
use crate::frame::upscale_targets::UpscaleTargets;
use crate::frame::vbao_history::VbaoHistory;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::host::render_rain::RenderRain;
use crate::lighting::ambient_gust::AmbientGust;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::ambient_occlusion_parameters::AmbientOcclusionParameters;
use crate::pass::ambient_occlusion_pass::AmbientOcclusionPass;
use crate::pass::ambient_occlusion_uniform::AmbientOcclusionUniform;
use crate::pass::bitmask_pass_names::BitmaskPassNames;
use crate::pass::bitmask_search::BitmaskSearch;
use crate::pass::bloom_parameters::BloomParameters;
use crate::pass::bloom_pass::BloomPass;
use crate::pass::bloom_uniform::BloomUniform;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::combine_parameters::CombineParameters;
use crate::pass::composited_parameters::CompositedParameters;
use crate::pass::contact_shadow_parameters::ContactShadowParameters;
use crate::pass::contact_shadow_pass::ContactShadowPass;
use crate::pass::contact_shadow_uniform::ContactShadowUniform;
use crate::pass::exposure_parameters::ExposureParameters;
use crate::pass::foliage_wind_values::FoliageWindValues;
use crate::pass::fsr_uniform::FsrUniform;
use crate::pass::fxaa_parameters::FxaaParameters;
use crate::pass::level_passes::LevelPasses;
use crate::pass::light_binning_parameters::LightBinningParameters;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::lights_parameters::LightsParameters;
use crate::pass::lights_uniform::LightsUniform;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::pyramid_depth_parameters::PyramidDepthParameters;
use crate::pass::pyramid_level_parameters::PyramidLevelParameters;
use crate::pass::rain_parameters::RainParameters;
use crate::pass::rain_uniform::RainUniform;
use crate::pass::reflection_parameters::ReflectionParameters;
use crate::pass::reflection_pass::ReflectionPass;
use crate::pass::reflection_trace::ReflectionTrace;
use crate::pass::reflection_uniform::ReflectionUniform;
use crate::pass::sky_haze_parameters::SkyHazeParameters;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::smaa_parameters::SmaaParameters;
use crate::pass::smaa_pass::SmaaPass;
use crate::pass::static_cull_parameters::StaticCullParameters;
use crate::pass::static_cull_params::StaticCullParams;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;
use crate::pass::sun_parameters::SunParameters;
use crate::pass::sun_shafts_parameters::SunShaftsParameters;
use crate::pass::temporal_parameters::TemporalParameters;
use crate::pass::temporal_uniform::TemporalUniform;
use crate::pass::thunder_parameters::ThunderParameters;
use crate::pass::thunder_uniform::ThunderUniform;
use crate::pass::upscale_parameters::UpscaleParameters;
use crate::pass::upscale_uniform::UpscaleUniform;
use crate::pass::vbao_parameters::VbaoParameters;
use crate::pass::vbao_pass::VbaoPass;
use crate::pass::vbao_uniform::VbaoUniform;
use crate::pass::view_binding::ViewBinding;
use crate::pass::water_draw::WaterDraw;
use crate::pass::wet_apply_parameters::WetApplyParameters;
use crate::pass::wet_patch_parameters::WetPatchParameters;
use crate::pass::wet_uniform::WetUniform;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::grass_level::GrassLevel;
use crate::scene::level::level_frame::LevelFrame;
use crate::scene::level::level_overlays::LevelOverlays;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::level_water::WaterFrame;
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::level::lights_frame::LightsFrame;
use crate::scene::level::lights_view::LightsView;
use crate::scene::level::scene_output::SceneOutput;
use crate::scene::level::scene_renderer::SceneRenderer;
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

/// The sway amplitude the shadows are redrawn at while the enhanced foliage motion blows: its largest move a metre of
/// reach, a tall crown's trunk swing over its height.
const FOLIAGE_SHADOW_AMPLITUDE: f32 = 0.02;

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
      renderer: SceneRenderer::new(device, view_layout, scene.statics.args.size()),
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
  }

  /// Sizes the targets to the viewport and writes what this frame's cull and lighting read.
  #[allow(clippy::too_many_arguments)]
  pub fn prepare(
    &mut self,
    scene: &mut LevelScene,
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

      self.state.targets = Some(targets);
      self.state.pyramid = Some(pyramid);
      self.state.temporal = None;
      self.state.fsr = None;
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
      .state
      .exposure
      .prepare(queue, &options.features.exposure, Instant::now());

    self.info.sun_sprite = self.renderer.flares.prepare(
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

    // The ambient effects blow the wind the grass, the rain and the campfires read this frame.
    let gust: AmbientGust = scene.gust;

    self.state.water.prepare(
      device,
      options,
      WaterFrame {
        targets: self.state.targets.as_ref(),
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
    self.prepare_rain(
      device,
      queue,
      passes,
      (lighting, weather),
      (options, gust, scene.started.elapsed().as_secs_f32()),
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
    self.prepare_bloom(options, lighting.engine);

    if !options.features.is_occlusion_culled {
      self.state.history = None;
    }

    // The engine's screen: the viewport's pixels, widened for a lens narrower than its 90 degrees.
    let screen: f32 =
      (width * height) as f32 * (90.0 / field_of_view.max(1.0)).powi(2) * (EPS_S + options.features.lod.geometry_lod);
    let threshold = |area: f32| -> f32 { (area / 3.0).powi(2) / screen };

    self.info.cull = StaticCullParams {
      cluster_count: scene.statics.get_cluster_count(),
      row_count: scene.statics.get_row_count(),
      batch_count: StaticBatch::COUNT as u32,
      impostor_count: scene.statics.get_impostor_count(),
      glod_start: threshold(options.features.lod.ssa_glod_start),
      glod_end: threshold(options.features.lod.ssa_glod_end),
      discard_below: options.features.lod.ssa_discard.powi(2) / screen,
      candidate_capacity: scene.statics.get_list_capacity(),
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
    );
    scene.lights.prepare(
      queue,
      &mut self.state.lights,
      LightsFrame {
        camera: view,
        settings: &options.features.lights,
        lod: (self.info.cull.glod_start, self.info.cull.glod_end),
        contents: scene.statics.get_contents(),
        sway: &to_sway(&scene.statics, self.info.sway),
        campfire_shares: &scene.campfire_shares,
      },
    );
    self.info.camera = *view;
    self.info.sun_direction = lighting.get_sun_direction();
    self.info.shadow_settings = options.features.shadows.clone();
    self.info.ambient_occlusion = options.features.ambient_occlusion;
    self.info.indirect_light = options.features.indirect_light;
    self.info.bitmask = BitmaskSearch::new(options);
    self.info.reflection_settings = options.features.reflections;
    self.info.reflection = ReflectionTrace::new(options, lighting);
    self.info.output = output;
    self.info.upscaling = options.output.upscaling;
    self.info.debug_view = options.mode.debug_view;
    self.info.is_wireframe = options.mode.is_wireframe;
    self.info.corrections = options.features.corrections;
    self.info.is_occlusion_drawn =
      options.mode.is_lit && !options.mode.is_wireframe && options.features.ambient_occlusion.is_enabled;
    self.prepare_temporal(device, view);
    self.prepare_smoothing(passes, options.features.antialiasing);
    self.prepare_upscale(device);
    self.info.lights_settings = options.features.lights;

    self.info.occlusion_settings = AmbientOcclusionUniform::new(
      &options.features.ambient_occlusion,
      view.projection,
      (width.div_ceil(2), height.div_ceil(2)),
    );
    self.prepare_bitmask(device, view.projection, (width.div_ceil(2), height.div_ceil(2)));
    self.prepare_reflections(device, (width, height));
    self.prepare_sorted(scene, device, queue, view);

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

    if self.info.is_temporal {
      self.state.noise_frame = self.state.noise_frame.wrapping_add(1);
    }

    // Marched only where the view is lit and solid: the sun's under its cascades, the lights' with or without them.
    let shadows: &RenderShadowSettings = &options.features.shadows;
    let is_marched: bool = options.mode.is_lit && !options.mode.is_wireframe;

    self.info.is_sun_contact = is_marched && shadows.is_sun_contact_drawn();
    self.info.contact_shadows = if is_marched {
      ContactShadowUniform::new(
        &shadows.contact,
        self.info.lighting.to_sun,
        height,
        self.state.noise_frame,
      )
    } else {
      ContactShadowUniform::default()
    };

    scene.particles.fill(view, options, &mut self.state.particles);

    self.state.particles.upload((device, queue));

    self.write_present(options);
    self.prepare_shadows(scene, device, queue, encoder, (view_layout, textures));
  }

  /// Readies this frame's shadows, sun, rain cover and lights, which the frame's passes then draw; `encoder` takes
  /// the copies a list growing makes.
  fn prepare_shadows(
    &mut self,
    scene: &LevelScene,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    (view_layout, textures): (&wgpu::BindGroupLayout, &TextureCache),
  ) {
    let SceneView { info, renderer, .. } = self;
    let SceneRenderer {
      shadows, rain_cover, ..
    } = renderer;
    let statics: &StaticScene = &scene.statics;
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
  }

  /// Readies the scene's light shadow faces every view of it asked for this frame, from this view's culling, once all
  /// were prepared; `encoder` takes the copies a list growing makes.
  pub fn prepare_scene_shadows(
    &self,
    scene: &mut LevelScene,
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    encoder: &mut wgpu::CommandEncoder,
    textures: &TextureCache,
  ) {
    let LevelScene { statics, lights, .. } = scene;
    let frame: ShadowFrame<'_> = ShadowFrame {
      scene: statics,
      camera: &self.info.camera,
      settings: &self.info.shadow_settings,
      sun_direction: self.info.sun_direction,
      sway: to_sway(statics, self.info.sway),
      params: &self.info.cull,
      textures,
    };

    lights.finish_shadows(device, queue, encoder, &frame);
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
      .record(move |context| passes.pyramid.record(context, pyramid, (&first, &levels)));
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
    (has_lights, is_occlusion_ambient): (bool, bool),
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

    let contact: UniformBinding<ContactShadowUniform> = runtime.push_uniform(&self.info.contact_shadows);
    // The contact shadows the sun multiplies its own by, or a lit texel where none are drawn.
    let contact_shadows: GraphTexture = if self.info.is_sun_contact {
      let (width, height) = targets.size;
      let marched: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
        "contact shadows",
        width,
        height,
        ContactShadowPass::FORMAT,
      ));
      let parameters: ContactShadowParameters = ContactShadowParameters {
        normal_target: targets.normal,
        depth_target: targets.depth,
        contact,
      };

      graph
        .add_raster_pass("contact shadows")
        .parameters(&parameters)
        .color(GraphColorAttachment::new(
          marched,
          wgpu::LoadOp::Clear(wgpu::Color::WHITE),
        ))
        .record(move |context| passes.contact_shadows.record(context, view, &parameters));

      marched
    } else {
      bindings.import_view(graph, "contact shadows lit", passes.contact_shadows.get_lit())
    };
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
      let is_contact: bool = self.info.contact_shadows.lights > 0;

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
    if is_occlusion_ambient && !self.info.ambient_occlusion.is_vbao() {
      let quality: RenderAmbientOcclusionQuality = self.info.ambient_occlusion.quality;
      let occlusion: UniformBinding<AmbientOcclusionUniform> = runtime.push_uniform(&self.info.occlusion_settings);

      for (stage, (name, (read, written))) in ["ambient occlusion", "occlusion denoise", "occlusion denoise back"]
        .into_iter()
        .zip(AmbientOcclusionPass::STAGES)
        .enumerate()
      {
        // Each stage reads the other target than it draws into, with the normals and the depth.
        let parameters: AmbientOcclusionParameters = AmbientOcclusionParameters {
          normal_target: targets.normal,
          depth_target: targets.depth,
          occlusion,
          source: targets.occlusion[1 - read],
        };

        graph
          .add_raster_pass(name)
          .parameters(&parameters)
          .color(GraphColorAttachment::new(
            targets.occlusion[written],
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .record(move |context| {
            passes
              .ambient_occlusion
              .record(context, (stage, quality), view, &parameters)
          });
      }
    }

    self
      .info
      .bitmask
      .and_then(|search| self.add_bitmask_passes((graph, bindings, runtime), passes, view, targets, search))
  }

  /// Declares the visibility-bitmask search, for VBAO's occlusion, the indirect light or both: the frame's light copied
  /// at half size while lit, searched (into the first occlusion target while it occludes), accumulated with the last
  /// frame's into the view's histories while it accumulates, and filtered (into the second occlusion target, which
  /// combine reads). Answers the indirect light filtered while lit.
  fn add_bitmask_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    targets: ViewTargetHandles,
    search: BitmaskSearch,
  ) -> Option<GraphTexture> {
    let quality: RenderAmbientOcclusionQuality = self.info.ambient_occlusion.quality;
    let names: BitmaskPassNames = search.get_names();
    let is_lit: bool = search.is_lit;
    let occlusion: UniformBinding<VbaoUniform> = runtime.push_uniform(&self.info.vbao);
    let empty: GraphTexture = bindings.import_view(graph, "vbao history empty", passes.vbao.get_empty());
    let dark: GraphTexture = bindings.import_view(graph, "indirect light none", passes.vbao.get_dark());
    let (width, height) = (targets.size.0.div_ceil(2), targets.size.1.div_ceil(2));
    let mut create = |label: &'static str, format: wgpu::TextureFormat| -> GraphTexture {
      graph.create_texture(GraphTextureDescriptor::new_2d(label, width, height, format))
    };
    // What a stage reads: the occlusion before it and its history, the light copied, and the light before it and its
    // history.
    let parameters =
      |(source, history): (GraphTexture, GraphTexture),
       (light_source, gathered, light_history): (GraphTexture, GraphTexture, GraphTexture)| {
        VbaoParameters {
          normal_target: targets.normal,
          depth_target: targets.depth,
          motion_target: targets.motion,
          occlusion,
          source,
          history,
          albedo_target: targets.albedo,
          material_target: targets.material,
          light_target: targets.light,
          light_source,
          gathered,
          light_history,
        }
      };
    let clear = |target: GraphTexture| GraphColorAttachment::new(target, wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT));
    // The occlusion targets while it occludes; a search of the light alone has its own, its visibility unused.
    let [searched, filtered] = if search.is_occluding {
      [targets.occlusion[1], targets.occlusion[0]]
    } else {
      [create("indirect search", ViewTargets::OCCLUSION), dark]
    };
    let light_source: GraphTexture = if is_lit {
      create("indirect source", VbaoPass::LIGHT)
    } else {
      dark
    };
    let lit_searched: Option<GraphTexture> = is_lit.then(|| create("indirect gathered", VbaoPass::LIGHT));
    let lit_filtered: Option<GraphTexture> = is_lit.then(|| create("indirect light", VbaoPass::LIGHT));

    if is_lit {
      let copy: VbaoParameters = parameters((empty, empty), (dark, dark, dark));

      graph
        .add_raster_pass(BitmaskSearch::SOURCE_PASS)
        .parameters(&copy)
        .color(clear(light_source))
        .record(move |context| passes.vbao.record_light_source(context, view, &copy));
    }

    let searching: VbaoParameters = parameters((empty, empty), (light_source, dark, dark));

    lit_searched
      .into_iter()
      .fold(
        graph
          .add_raster_pass(names.search)
          .parameters(&searching)
          .color(clear(searched)),
        |builder, light| builder.color(clear(light)),
      )
      .record(move |context| passes.vbao.record_search(context, (quality, is_lit), view, &searching));

    // The last frame's accumulations and this frame's, while it accumulates.
    let accumulated: bool = self.info.is_bitmask_accumulated;
    let histories: Option<[GraphTexture; 2]> = self.state.occlusion.as_ref().filter(|_| accumulated).map(|history| {
      [
        bindings.import_view(graph, "vbao history", &history.views[1 - history.index]),
        bindings.import_view(graph, "vbao history", &history.views[history.index]),
      ]
    });
    let light_histories: Option<[GraphTexture; 2]> = self
      .state
      .indirect
      .as_ref()
      .filter(|_| accumulated && is_lit)
      .map(|history| {
        [
          bindings.import_view(graph, "indirect light history", &history.views[1 - history.index]),
          bindings.import_view(graph, "indirect light history", &history.views[history.index]),
        ]
      });
    let (gathered, gathered_light): (GraphTexture, Option<GraphTexture>) = match histories {
      Some([previous, written]) => {
        let [previous_light, written_light] = light_histories.unwrap_or([dark, dark]);
        let written_light: Option<GraphTexture> = light_histories.map(|_| written_light);
        let accumulate: VbaoParameters = parameters(
          (searched, previous),
          (dark, lit_searched.unwrap_or(dark), previous_light),
        );
        let is_lit_accumulated: bool = written_light.is_some();

        written_light
          .into_iter()
          .fold(
            graph
              .add_raster_pass(names.accumulate)
              .parameters(&accumulate)
              .color(clear(written)),
            |builder, light| builder.color(clear(light)),
          )
          .record(move |context| {
            passes
              .vbao
              .record_accumulate(context, is_lit_accumulated, view, &accumulate)
          });

        (written, written_light.or(lit_searched))
      }
      None => (searched, lit_searched),
    };
    let filtering: VbaoParameters = parameters((gathered, empty), (dark, gathered_light.unwrap_or(dark), dark));

    search
      .is_occluding
      .then_some(filtered)
      .into_iter()
      .chain(lit_filtered)
      .fold(
        graph.add_raster_pass(names.filter).parameters(&filtering),
        |builder, target| builder.color(clear(target)),
      )
      .record(move |context| passes.vbao.record_filter(context, search, view, &filtering));

    lit_filtered
  }

  /// Makes the bitmask search's histories while it accumulates, the light's while it is lit too, dropping them
  /// otherwise, and writes what its passes read, over the search's half size.
  fn prepare_bitmask(&mut self, device: &wgpu::Device, projection: Mat4, (width, height): (u32, u32)) {
    let settings: RenderAmbientOcclusionSettings = self.info.ambient_occlusion;
    let is_lit: bool = self.info.bitmask.is_some_and(|search| search.is_lit);
    let is_accumulated: bool = self.info.bitmask.is_some() && settings.vbao.get_accumulation() > 1;

    self.info.is_bitmask_accumulated = is_accumulated;

    if !is_accumulated
      || self
        .state
        .occlusion
        .as_ref()
        .is_some_and(|history| !history.is_sized(width, height))
    {
      self.state.occlusion = None;
    }

    if !(is_accumulated && is_lit)
      || self
        .state
        .indirect
        .as_ref()
        .is_some_and(|history| !history.is_sized(width, height))
    {
      self.state.indirect = None;
    }

    if is_accumulated && is_lit && self.state.indirect.is_none() {
      self.state.indirect = Some(IndirectLightHistory::new(device, width, height));
      // Both start afresh, so the light's frames are counted from its own first.
      self.state.occlusion = None;
    }

    let history: Option<&VbaoHistory> = if is_accumulated {
      Some(
        self
          .state
          .occlusion
          .get_or_insert_with(|| VbaoHistory::new(device, width, height)),
      )
    } else {
      None
    };
    let has_light_history: bool = self.state.indirect.as_ref().is_some_and(|it| it.is_valid);

    self.info.vbao = VbaoUniform::new(
      &settings,
      if is_lit {
        (self.info.indirect_light.intensity, self.info.indirect_light.radius)
      } else {
        (0.0, 0.0)
      },
      projection,
      (width, height),
      history.map_or(0, |it| it.frame),
      (history.is_some_and(|it| it.is_valid), has_light_history),
    );
  }

  /// Declares the screen-space reflections while they are traced: the depth reduced to its nearest at the quality's
  /// size, every glossy pixel's ray marched over it, accumulated with the last frame's into the view's history, and filtered.
  /// Answers the reflections filtered, none where they are not traced.
  fn add_reflection_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (targets, lit): (ViewTargetHandles, LightingHandles<'a>),
    sky: &SkyParameters<'a>,
  ) -> Option<GraphTexture> {
    self.info.reflection?;

    let (Some(depth), Some(history)) = (&self.state.reflection_depth, &self.state.reflections) else {
      return None;
    };
    let sky: SkyParameters<'a> = *sky;
    let reflection: UniformBinding<ReflectionUniform> = runtime.push_uniform(&self.info.reflections);
    let empty: GraphTexture = bindings.import_view(graph, "reflections empty", passes.reflections.get_empty());
    let nearest: GraphTexture = bindings.import_view(graph, "reflection depth", &depth.view);
    let reduction: PyramidDepthParameters = PyramidDepthParameters {
      source_depth: targets.depth,
      target_level: nearest,
    };

    graph
      .add_compute_pass(ReflectionTrace::DEPTH_PASS)
      .parameters(&reduction)
      .record(move |context| passes.reflections.record_depth(context, depth, &reduction));

    let (width, height) = (depth.width, depth.height);
    let mut create = |label: &'static str, format: wgpu::TextureFormat| -> GraphTexture {
      graph.create_texture(GraphTextureDescriptor::new_2d(label, width, height, format))
    };
    let traced: GraphTexture = create("reflections traced", ReflectionPass::TRACED);
    let traced_length: GraphTexture = create("reflections length", ReflectionPass::LENGTH);
    let filtered: GraphTexture = create("reflections", ReflectionPass::TRACED);
    // What a stage reads: the stage before's result and its rays' lengths, and the last frame's accumulation.
    let parameters = |(source, lengths): (GraphTexture, GraphTexture),
                      (previous, held): (GraphTexture, GraphTexture)| {
      ReflectionParameters {
        albedo_target: targets.albedo,
        normal_target: targets.normal,
        material_target: targets.material,
        depth_target: targets.depth,
        light_target: targets.light,
        motion_target: targets.motion,
        occlusion_target: targets.occlusion[0],
        material_lut: lit.material_lut,
        lut_sampler: lit.lut_sampler,
        lighting: lit.lighting,
        reflection,
        nearest_depth: nearest,
        traced: source,
        traced_length: lengths,
        history: previous,
        history_held: held,
      }
    };
    let clear = |target: GraphTexture| GraphColorAttachment::new(target, wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT));
    let tracing: ReflectionParameters = parameters((empty, empty), (empty, empty));

    graph
      .add_raster_pass(ReflectionTrace::TRACE_PASS)
      .parameters(&tracing)
      .parameters(&sky)
      .color(clear(traced))
      .color(clear(traced_length))
      .record(move |context| passes.reflections.record_trace(context, view, &tracing, &sky));

    let index: usize = history.index;
    let [previous, written] =
      [1 - index, index].map(|at| bindings.import_view(&mut *graph, "reflection history", &history.colour_views[at]));
    let [previous_held, written_held] = [1 - index, index]
      .map(|at| bindings.import_view(&mut *graph, "reflection history held", &history.held_views[at]));
    let accumulating: ReflectionParameters = parameters((traced, traced_length), (previous, previous_held));

    graph
      .add_raster_pass(ReflectionTrace::ACCUMULATE_PASS)
      .parameters(&accumulating)
      .color(clear(written))
      .color(clear(written_held))
      .record(move |context| passes.reflections.record_accumulate(context, view, &accumulating));

    let filtering: ReflectionParameters = parameters((written, empty), (empty, empty));

    graph
      .add_raster_pass(ReflectionTrace::FILTER_PASS)
      .parameters(&filtering)
      .color(clear(filtered))
      .record(move |context| passes.reflections.record_filter(context, view, &filtering));

    Some(filtered)
  }

  /// Makes the nearest depth and the history at the size the reflections trace at while they are traced,
  /// dropping both otherwise, and writes what their passes read.
  fn prepare_reflections(&mut self, device: &wgpu::Device, (width, height): (u32, u32)) {
    let Some(trace) = self.info.reflection else {
      self.state.reflection_depth = None;
      self.state.reflections = None;

      return;
    };
    let ratio: u32 = trace.get_ratio();
    let (traced_width, traced_height) = (width.div_ceil(ratio), height.div_ceil(ratio));

    if !self
      .state
      .reflection_depth
      .as_ref()
      .is_some_and(|it| it.is_sized(traced_width, traced_height))
    {
      self.state.reflection_depth = Some(ReflectionDepth::new(device, traced_width, traced_height));
    }

    if !self
      .state
      .reflections
      .as_ref()
      .is_some_and(|it| it.is_sized(traced_width, traced_height))
    {
      self.state.reflections = Some(ReflectionHistory::new(device, traced_width, traced_height));
    }

    let history: Option<&ReflectionHistory> = self.state.reflections.as_ref();

    self.info.reflections = ReflectionUniform::new(
      &self.info.reflection_settings,
      (width, height),
      history.map_or(0, |it| it.frame),
      history.is_some_and(|it| it.is_valid),
    );
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

  /// Ends a frame its graph recorded: the water's reflection, the bitmask search's accumulations and the temporal
  /// resolve's history it wrote become the ones the next frame keeps.
  pub fn end_frame(&mut self, frame: &LevelFrame) {
    if frame.is_lit {
      self.state.water.finish_frame();

      if self.info.is_bitmask_accumulated {
        if let Some(history) = &mut self.state.occlusion {
          history.swap();
        }

        if let Some(history) = &mut self.state.indirect {
          history.swap();
        }
      }

      if self.info.reflection.is_some()
        && let Some(history) = &mut self.state.reflections
      {
        history.swap();
      }
    }

    match frame.resolve {
      Some("fsr2") => {
        if let Some(fsr) = &mut self.state.fsr {
          fsr.swap();
        }
      }
      Some("temporal") => {
        if let Some(history) = &mut self.state.temporal {
          history.swap();
        }
      }
      _ => {}
    }
  }

  /// Notes what the present pass reads, once the frame knows what draws into the distortion target: `def_distort`
  /// while the water or a particle does, nothing otherwise.
  fn write_present(&mut self, options: &RenderViewOptions) {
    let Some(targets) = &self.state.targets else {
      return;
    };
    let water: &RenderWaterSettings = &options.features.water;
    let is_water_distorting: bool = water.is_enabled && water.is_distorted && options.mode.is_lit;
    let is_distorting: bool =
      !options.mode.is_wireframe && (is_water_distorting || self.state.particles.is_distorting());

    self.info.present = PresentUniform::new(
      self.info.debug_view,
      (
        self.info.is_occlusion_drawn,
        self.info.bitmask.is_some_and(|search| search.is_lit),
        self.info.reflection.map(ReflectionTrace::get_ratio),
      ),
      !targets.is_sized(self.info.output.width, self.info.output.height),
      if is_distorting { water.distortion } else { 0.0 },
      self.info.output,
      &self.info.corrections,
      (self.info.selection_color, self.info.is_bloomed),
    );
  }

  /// Notes this frame's bloom while the view blooms (`phase_bloom`): the build's threshold over the frame's size, and
  /// the blur across and down, down by the frame's height over its width, one-sided on Monolith as Anomaly's
  /// `bloom_filter.ps` reads it.
  fn prepare_bloom(&mut self, options: &RenderViewOptions, engine: XrayEngine) {
    let bloom: &RenderBloomSettings = &options.features.bloom;

    self.info.is_bloomed =
      bloom.is_enabled && options.mode.is_lit && !options.mode.is_wireframe && self.state.targets.is_some();

    let Some(targets) = self.state.targets.as_ref().filter(|_| self.info.is_bloomed) else {
      return;
    };

    let size: (u32, u32) = (targets.width, targets.height);
    let aspect: f32 = size.1 as f32 / size.0 as f32;
    let is_one_sided: bool = engine == XrayEngine::Extended;

    self.info.bloom = [
      BloomUniform::build(size, bloom.threshold),
      BloomUniform::filter(true, (bloom.radius, bloom.strength), aspect, is_one_sided),
      BloomUniform::filter(false, (bloom.radius, bloom.strength), aspect, is_one_sided),
    ];
  }

  /// Writes this frame's rain, while the weather rains and the view shows it: the splash's model built for the
  /// level's weather, the uniform at the cover last drawn, and what the draw binds.
  fn prepare_rain(
    &mut self,
    device: &wgpu::Device,
    _queue: &wgpu::Queue,
    _passes: LevelPasses<'_>,
    (lighting, weather): (&RenderLighting, Option<&Arc<RenderLevelWeather>>),
    (options, gust, time): (&RenderViewOptions, AmbientGust, f32),
    weather_textures: &WeatherTextureCache,
  ) {
    self.info.rain_draw = None;
    self.info.weather_views.rain = None;
    self.info.weather_views.wet = None;

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

    if self.renderer.splash.as_ref().is_none_or(|(built, _)| *built != key) {
      self.renderer.splash = Some((key, WeatherModelBuffers::new(device, rain.drop.as_ref())));
    }

    let Some((_, splash)) = &self.renderer.splash else {
      return;
    };
    let uniform: RainUniform = RainUniform::new(
      &rainfall,
      (lighting.wind, gust.strength),
      self.renderer.rain_cover.get_window(),
      time,
      splash.index_count,
    );
    let flat = |reference: Option<&str>| weather_textures.get_view(reference, WeatherTextureKind::Flat).clone();

    self.info.rain = uniform;
    self.info.weather_views.rain = Some([
      flat(Some(&rain.streak)),
      flat(rain.drop.as_ref().map(|drop| drop.texture.as_str())),
    ]);
    self.info.rain_draw = Some((uniform.count, splash.index_count));

    let Some(wet) = weather.and_then(|weather| weather.wet.as_ref()) else {
      return;
    };
    self.info.wet = WetUniform {
      density: rainfall.density.clamp(0.0, 1.0),
      time: uniform.time,
      is_extended: (lighting.engine == XrayEngine::Extended) as u32 as f32,
      _pad: 0.0,
      window: uniform.window,
    };
    self.info.weather_views.wet = Some([
      weather_textures
        .get_view(Some(&wet.splash), WeatherTextureKind::Volume)
        .clone(),
      weather_textures
        .get_view(Some(&wet.flow), WeatherTextureKind::Flat)
        .clone(),
    ]);
  }

  /// Writes this frame's strike, while a bolt strikes and the view shows it: every bolt model built for the level's
  /// weather, the uniform where the strike stands, and what its model and glows bind.
  fn prepare_thunder(
    &mut self,
    device: &wgpu::Device,
    _queue: &wgpu::Queue,
    _passes: LevelPasses<'_>,
    (lighting, weather): (&RenderLighting, Option<&Arc<RenderLevelWeather>>),
    options: &RenderViewOptions,
    weather_textures: &WeatherTextureCache,
  ) {
    self.info.thunder_draw = None;
    self.info.weather_views.thunder = None;

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
      .renderer
      .thunder_models
      .as_ref()
      .is_none_or(|(built, _)| *built != key)
    {
      let models: Vec<WeatherModelBuffers> = thunder
        .models
        .iter()
        .map(|model| WeatherModelBuffers::new(device, Some(&model.mesh)))
        .collect();

      self.renderer.thunder_models = Some((key, models));
    }

    let Some((_, models)) = &self.renderer.thunder_models else {
      return;
    };
    let model = bolt
      .model
      .and_then(|index| Some((models.get(index)?, thunder.models.get(index)?)));
    let flat = |reference: &str| {
      weather_textures
        .get_view(Some(reference), WeatherTextureKind::Flat)
        .clone()
    };
    let model_texture: &str = model.map_or("", |(_, model)| model.mesh.texture.as_str());

    self.info.thunder = ThunderUniform::new(strike);
    self.info.weather_views.thunder = Some((
      [flat(model_texture), flat(&bolt.top.texture), flat(&bolt.center.texture)],
      bolt.model.filter(|_| model.is_some()),
    ));

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
      is_scene_first: false,
      is_lit: self.state.targets.is_some(),
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
    scene: &'a LevelScene,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    textures: &'a TextureCache,
    frame: &LevelFrame,
  ) -> Option<SceneOutput> {
    let is_occluding: bool = self.info.cull.is_occluding != 0;
    let pick_slot: Option<usize> = frame.pick_slot;
    let resolve: Option<&'static str> = frame.resolve;
    let is_drawn: bool = !self.info.is_wireframe;
    let is_wallmarked: bool = self.info.is_wallmarked && is_drawn;
    let is_raining: bool = self.info.rain_draw.is_some();
    let is_wet: bool = is_raining && self.info.weather_views.wet.is_some();
    let is_lit: bool = frame.is_lit;
    let has_lights: bool = self.state.lights.get_count() > 0;
    let is_occlusion_ambient: bool = self.info.is_occlusion_drawn;
    let has_sky: bool = self.info.sky.is_some();
    let is_hazing: bool = has_sky && self.info.is_hazing;
    let is_composited: bool = is_drawn && has_sky;
    let has_particles: bool = is_drawn && self.state.particles.is_drawing();
    let is_shafted: bool = self.info.is_shafted;
    let is_rain_drawn: bool = is_raining && self.info.weather_views.rain.is_some();
    let is_thundering: bool = self.info.thunder_draw.is_some() && self.info.weather_views.thunder.is_some();
    let is_bloomed: bool = self.info.is_bloomed;
    let particle_surfaces: &wgpu::Buffer = scene.particles.get_surfaces();
    let smoothing: Option<RenderAntialiasing> = self.info.smoothing;
    let is_sharpened: bool = self.state.upscale.is_some() && self.info.upscaling.is_sharpened();
    let is_adapting: bool = self.state.exposure.is_adapting();
    let texture_group: &wgpu::BindGroup = textures.get_bind_group();
    let scene_view: &'a SceneView = self;
    let (Some(targets), Some(pyramid)) = (&scene_view.state.targets, &scene_view.state.pyramid) else {
      return None;
    };
    let unlit: GraphTexture = bindings.import_view(&mut *graph, "indirect light none", passes.vbao.get_dark());
    let untraced: GraphTexture =
      bindings.import_view(&mut *graph, "reflections none", passes.reflections.get_untraced());
    let handles: ViewTargetHandles = ViewTargetHandles::import(&mut *graph, &mut *bindings, targets);
    // The resolved frame goes where the present pass reads it: the upscaled frame, or the scene drawn at its size.
    let upscaled: Option<[GraphTexture; 2]> = scene_view.state.upscale.as_ref().map(|upscale| {
      [
        bindings.import_view(&mut *graph, "upscaled", &upscale.views[0]),
        bindings.import_view(&mut *graph, "sharpened", &upscale.views[1]),
      ]
    });
    let upscale: UniformBinding<UpscaleUniform> = runtime.push_uniform(&scene_view.info.upscale);
    let statics: StaticSceneHandles = StaticSceneHandles::import(
      (&mut *graph, &mut *bindings, runtime),
      &scene.statics,
      &pyramid.view,
      (&scene_view.info.cull, &scene_view.info.occlusion, &scene_view.info.wind),
    );
    let layouts: [StaticDrawParameters; StaticLayout::COUNT] = statics.get_camera_draws().layouts;
    let lit: LightingHandles = LightingHandles {
      lighting: runtime.push_uniform(&scene_view.info.lighting),
      exposure: bindings.import_buffer(&mut *graph, "exposure", &scene_view.state.exposure.state),
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
          .record(context.get_encoder(), args, StaticScene::STATS_OFFSET);
      });

    if let (true, Some(grass_args)) = (is_drawn, grass_args) {
      scene_view.state.grass.add_draw(
        &mut *graph,
        (passes.grass, grass_level),
        (handles, grass_args),
        (view, texture_group),
      );
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

    // The rain's cover, which its passes draw and the rain and the wet surfaces read.
    let cover: Option<GraphTexture> =
      is_raining.then(|| bindings.import_view(&mut *graph, "rain cover", &scene_view.renderer.rain_cover.depth));
    let wet: Option<UniformBinding<WetUniform>> = is_wet.then(|| runtime.push_uniform(&scene_view.info.wet));

    if let Some(cover) = cover {
      scene_view.renderer.rain_cover.add_passes(
        (&mut *graph, &mut *bindings),
        passes,
        (&statics, cover),
        (&scene_view.info.cull, texture_group),
      );
    }

    if frame.is_scene_first {
      scene.lights.add_shadow_passes(
        (&mut *graph, &mut *bindings),
        passes,
        &statics,
        (&scene_view.info.cull, texture_group),
      );
    }

    graph.begin_group("lighting");

    // The rain wets the G-buffer before any light is drawn over it.
    if let (Some(cover), Some(wet), Some([splash, flow])) = (cover, wet, &scene_view.info.weather_views.wet) {
      let patch: WetPatchParameters = WetPatchParameters {
        depth_target: handles.depth,
        albedo_target: handles.albedo,
        normal_target: handles.normal,
        cover,
        splash: bindings.import_view(&mut *graph, "wet splash", splash),
        flow: bindings.import_view(&mut *graph, "wet flow", flow),
        wet_sampler: passes.wet.get_sampler(),
        wet,
      };
      let apply: WetApplyParameters = WetApplyParameters {
        depth_target: handles.depth,
        patched: handles.light,
        wet,
      };

      // The patches go into the light, which the wet look over the normals and the albedo then reads.
      graph
        .add_raster_pass("wet patch")
        .parameters(&patch)
        .color(GraphColorAttachment::new(handles.light, wgpu::LoadOp::Load))
        .record(move |context| passes.wet.record(context, 0, view, &patch));

      for (stage, (name, target)) in [("wet normal", handles.normal), ("wet albedo", handles.albedo)]
        .into_iter()
        .enumerate()
      {
        graph
          .add_raster_pass(name)
          .parameters(&apply)
          .color(GraphColorAttachment::new(target, wgpu::LoadOp::Load))
          .record(move |context| passes.wet.record(context, stage + 1, view, &apply));
      }
    }

    // The indirect light combine adds and a debug view shows, or a texel of none; and the reflections traced.
    let mut indirect_light: Option<GraphTexture> = None;
    let mut reflections: Option<GraphTexture> = None;

    if is_lit {
      indirect_light = scene_view.add_lighting_passes(
        (&mut *graph, &mut *bindings, runtime),
        passes,
        (view, texture_group, scene),
        (handles, lit),
        (has_lights, is_occlusion_ambient),
      );

      let sky: Option<SkyParameters> = scene_view
        .info
        .sky
        .as_ref()
        .map(|views| passes.sky.import(views, &mut *graph, &mut *bindings));

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

      if let Some(sky) = sky {
        reflections = scene_view.add_reflection_passes(
          (&mut *graph, &mut *bindings, runtime),
          passes,
          view,
          (handles, lit),
          &sky,
        );

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

      // FSR 2's reactive mask is what the water and the blended surfaces change of the frame drawn so far.
      if let Some(fsr) = &scene_view.state.fsr {
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

      if let (true, Some(skies)) = (
        scene_view.state.water.is_drawn(),
        scene_view.info.sky.as_ref().map(|sky| &sky.cubes),
      ) {
        let draw: WaterDraw<'_> = WaterDraw {
          targets: handles,
          view,
          textures: texture_group,
          layouts,
          args: Self::list_draw_args(&statics, &scene_view.info.cull),
          lighting: lit.lighting,
          skies: [&skies[0], &skies[1]],
          sky_sampler: passes.sky.get_clamp(),
          water: &scene_view.state.water,
        };

        passes.water.add_passes(&mut *graph, &mut *bindings, runtime, draw);
      }

      graph.begin_group("post");

      if let (true, Some(sky)) = (is_composited, sky) {
        let args: Vec<GraphBuffer> = Self::list_draw_args(&statics, &scene_view.info.cull);
        // The models' composited clusters, back to front, which the view lists itself.
        let sorted: Option<StaticDrawParameters> = scene_view
          .state
          .sorted_list
          .as_ref()
          .filter(|_| scene_view.info.sorted_count > 0)
          .map(|list| {
            let list: GraphBuffer = bindings.import_buffer(&mut *graph, "sorted composited", list);

            statics.get_layout_draws(StorageArray::new(list))[StaticLayout::Model.get_index()]
          });
        let sorted_count: u32 = scene_view.info.sorted_count;
        let builder = args.iter().fold(
          StaticDraws::declare_layouts(Self::add_over_scene(&mut *graph, handles, "composited"), &layouts),
          |builder, args| builder.buffer(*args, GraphBufferAccess::Indirect),
        );
        let composited: CompositedParameters = CompositedParameters {
          lighting: lit.lighting,
          exposure: StorageValue::new(lit.exposure),
          material_lut: lit.material_lut,
          lut_sampler: lit.lut_sampler,
          shadow_maps: lit.shadow_maps,
          shadows: lit.shadows,
        };
        let builder = match &sorted {
          Some(sorted) => builder.parameters(sorted),
          None => builder,
        }
        .parameters(&composited)
        .parameters(&sky);

        builder.record(move |context| {
          let args: Vec<&wgpu::Buffer> = args.iter().map(|args| context.get_buffer(*args)).collect();

          passes.composited.record(
            context,
            (view, &layouts, texture_group),
            (&composited, &sky),
            &args,
            (sorted.as_ref(), sorted_count),
          );
        });
      }

      if has_particles {
        scene_view.state.particles.add_passes(
          (&mut *graph, &mut *bindings),
          passes.particles,
          (handles, lit.lighting, particle_surfaces),
          (view, texture_group),
        );
      }

      if is_shafted {
        let parameters: SunShaftsParameters = SunShaftsParameters {
          depth_target: handles.depth,
          shadow_maps: lit.shadow_maps,
          shadows: lit.shadows,
          lighting: lit.lighting,
          exposure: StorageValue::new(lit.exposure),
        };

        graph
          .add_raster_pass("sun shafts")
          .parameters(&parameters)
          .color(GraphColorAttachment::new(handles.scene, wgpu::LoadOp::Load))
          .color(GraphColorAttachment::new(handles.high, wgpu::LoadOp::Load))
          .record(move |context| passes.sun_shafts.record(context, view, &parameters));
      }

      if let (true, Some(counts), Some(cover), Some([streak, splash]), Some((_, model))) = (
        is_rain_drawn,
        scene_view.info.rain_draw,
        cover,
        &scene_view.info.weather_views.rain,
        &scene_view.renderer.splash,
      ) {
        let parameters: RainParameters = RainParameters {
          rain: runtime.push_uniform(&scene_view.info.rain),
          cover,
          streak_texture: bindings.import_view(&mut *graph, "rain streak", streak),
          splash_texture: bindings.import_view(&mut *graph, "rain splash", splash),
          rain_sampler: passes.rain.get_sampler(),
          splash_vertices: StorageArray::new(bindings.import_buffer(&mut *graph, "splash vertices", &model.vertices)),
          splash_indices: StorageArray::new(bindings.import_buffer(&mut *graph, "splash indices", &model.indices)),
        };

        Self::add_over_scene(&mut *graph, handles, "rain")
          .parameters(&parameters)
          .record(move |context| passes.rain.record(context, view, &parameters, counts));
      }

      if let (true, Some(draws), Some((textures, model))) = (
        is_thundering,
        scene_view.info.thunder_draw,
        &scene_view.info.weather_views.thunder,
      ) {
        let thunder: UniformBinding<ThunderUniform> = runtime.push_uniform(&scene_view.info.thunder);
        let no_model: &WeatherModelBuffers = &scene_view.renderer.no_model;
        let bolt: &WeatherModelBuffers = model
          .and_then(|index| scene_view.renderer.thunder_models.as_ref()?.1.get(index))
          .unwrap_or(no_model);
        // The bolt's model with its texture, then each glow's quad with its own.
        let entries: [ThunderParameters; 3] =
          [(0, bolt), (1, no_model), (2, no_model)].map(|(entry, buffers)| ThunderParameters {
            thunder,
            thunder_texture: bindings.import_view(&mut *graph, "thunder texture", &textures[entry]),
            thunder_sampler: passes.thunder.get_sampler(),
            model_vertices: StorageArray::new(bindings.import_buffer(
              &mut *graph,
              "thunder vertices",
              &buffers.vertices,
            )),
            model_indices: StorageArray::new(bindings.import_buffer(&mut *graph, "thunder indices", &buffers.indices)),
          });

        entries
          .iter()
          .fold(
            Self::add_over_scene(&mut *graph, handles, "thunder"),
            |builder, parameters| builder.parameters(parameters),
          )
          .record(move |context| passes.thunder.record(context, view, &entries, draws));
      }

      scene_view.renderer.flares.add_passes(
        (&mut *graph, &mut *bindings, runtime),
        passes.flares,
        (handles, lit),
        view,
      );

      if is_bloomed {
        // Built from the high target into the first, blurred across into the second, then down into the first.
        for (stage, (read, written)) in [
          (handles.high, handles.bloom[0]),
          (handles.bloom[0], handles.bloom[1]),
          (handles.bloom[1], handles.bloom[0]),
        ]
        .into_iter()
        .enumerate()
        {
          let parameters: BloomParameters = BloomParameters {
            source: read,
            source_sampler: passes.bloom.get_sampler(),
            bloom: runtime.push_uniform(&scene_view.info.bloom[stage]),
          };

          graph
            .add_raster_pass(BloomPass::STAGES[stage])
            .parameters(&parameters)
            .color(GraphColorAttachment::new(
              written,
              wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            ))
            .record(move |context| passes.bloom.record(context, stage, &parameters));
        }
      }

      if let Some(mode) = smoothing {
        let sized = |label: &'static str, format: wgpu::TextureFormat| {
          GraphTextureDescriptor::new_2d(label, targets.width, targets.height, format)
        };
        let target: GraphTexture = graph.create_texture(sized("smoothed", ViewTargets::SCENE));

        match (mode, passes.smaa) {
          (RenderAntialiasing::Smaa, Some(pass)) => {
            let edges: GraphTexture = graph.create_texture(sized("smaa edges", SmaaPass::TARGET_FORMAT));
            let weights: GraphTexture = graph.create_texture(sized("smaa weights", SmaaPass::TARGET_FORMAT));
            let [area, search, empty] = pass.get_lookups();
            let [area, search] = [("smaa area", area), ("smaa search", search)]
              .map(|(label, view)| bindings.import_view(&mut *graph, label, view));
            // Each stage reads an empty texture where it would read what it writes or what comes after it.
            let [no_edges, no_weights] =
              ["smaa no edges", "smaa no weights"].map(|label| bindings.import_view(&mut *graph, label, empty));
            let [linear_sampler, point_sampler] = pass.get_samplers();

            for (stage, (name, (edges_texture, weights_texture), written)) in [
              ("smaa edges", (no_edges, no_weights), edges),
              ("smaa weights", (edges, no_weights), weights),
              ("smaa", (edges, weights), target),
            ]
            .into_iter()
            .enumerate()
            {
              let parameters: SmaaParameters = SmaaParameters {
                source: handles.scene,
                edges_texture,
                weights_texture,
                area_texture: area,
                search_texture: search,
                linear_sampler,
                point_sampler,
              };

              graph
                .add_raster_pass(name)
                .parameters(&parameters)
                .color(GraphColorAttachment::new(
                  written,
                  wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                ))
                .record(move |context| pass.record(context, stage, &parameters));
            }
          }
          _ => {
            let parameters: FxaaParameters = FxaaParameters {
              frame: handles.scene,
              frame_sampler: passes.fxaa.get_sampler(),
            };

            graph
              .add_raster_pass("fxaa")
              .parameters(&parameters)
              .color(GraphColorAttachment::new(
                target,
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
              ))
              .record(move |context| passes.fxaa.record(context, &parameters));
          }
        }

        Self::add_copy(&mut *graph, "smoothed copy", target, handles.scene);
      }

      let resolved: GraphTexture = upscaled.map_or(handles.scene, |[upscaled, _]| upscaled);

      match (
        resolve,
        &scene_view.state.fsr,
        &scene_view.state.temporal,
        &scene_view.state.upscale,
      ) {
        (Some("fsr2"), Some(fsr), _, _) => {
          let history: GraphTexture = bindings.import_view(&mut *graph, "fsr2 history", &fsr.history[fsr.index]);

          passes.fsr.add_passes(
            (&mut *graph, &mut *bindings, runtime),
            handles,
            (fsr, &scene_view.info.fsr),
          );
          Self::add_copy(&mut *graph, "fsr2 output", history, resolved);
        }
        (Some("temporal"), _, Some(history), _) => {
          let index: usize = history.index;
          let target: GraphTexture = bindings.import_view(&mut *graph, "temporal history", &history.views[index]);
          let parameters: TemporalParameters = TemporalParameters {
            frame: handles.scene,
            depth_target: handles.depth,
            history: bindings.import_view(&mut *graph, "temporal history", &history.views[1 - index]),
            history_sampler: passes.temporal.get_sampler(),
            temporal: runtime.push_uniform(&scene_view.info.temporal),
            motion_target: handles.motion,
          };

          graph
            .add_raster_pass("temporal")
            .parameters(&parameters)
            .color(GraphColorAttachment::new(
              target,
              wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            ))
            .record(move |context| passes.temporal.record(context, view, &parameters));
          Self::add_copy(&mut *graph, "temporal output", target, resolved);
        }
        (Some(_), _, _, Some(_)) => {
          if let Some([upscaled, _]) = upscaled {
            let parameters: UpscaleParameters = UpscaleParameters {
              source: handles.scene,
              upscale,
            };

            graph
              .add_raster_pass("upscale")
              .parameters(&parameters)
              .color(GraphColorAttachment::new(
                upscaled,
                wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
              ))
              .record(move |context| passes.upscale.record(context, 0, &parameters));
          }
        }
        _ => {}
      }

      if let (true, Some([upscaled, sharpened])) = (is_sharpened, upscaled) {
        let parameters: UpscaleParameters = UpscaleParameters {
          source: upscaled,
          upscale,
        };

        graph
          .add_raster_pass("sharpen")
          .parameters(&parameters)
          .color(GraphColorAttachment::new(
            sharpened,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .record(move |context| passes.upscale.record(context, 1, &parameters));
      }

      if is_adapting {
        let parameters: ExposureParameters = ExposureParameters {
          high: handles.high,
          state: StorageValueMut::new(lit.exposure),
          params: runtime.push_uniform(&scene_view.state.exposure.params),
        };

        graph
          .add_compute_pass("exposure")
          .parameters(&parameters)
          .record(move |context| passes.exposure.record(context, &parameters));
      }
    }

    if let (Some(slot), Some(target), Some(pick_view)) =
      (pick_slot, &scene_view.state.pick_target, &scene_view.state.pick_view)
    {
      scene_view.add_pick_passes(
        (&mut *graph, &mut *bindings),
        passes.gbuffer,
        (pick_view, texture_group, &statics),
        (target, slot),
      );
    }

    Some(SceneOutput {
      targets: handles,
      shown: upscaled.map_or(
        handles.scene,
        |[upscaled, sharpened]| {
          if is_sharpened { sharpened } else { upscaled }
        },
      ),
      present: runtime.push_uniform(&scene_view.info.present),
      lighting: lit.lighting,
      indirect_light: indirect_light.unwrap_or(unlit),
      reflections: reflections.unwrap_or(untraced),
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
  fn prepare_temporal(&mut self, device: &wgpu::Device, view: &CameraView) {
    if self.state.targets.is_none() || !self.info.is_temporal || self.info.is_fsr {
      self.state.temporal = None;
      self.state.temporal_previous = None;
      self.prepare_fsr(device, view);

      return;
    }

    self.state.fsr = None;
    let (width, height): (u32, u32) = (self.info.output.width.max(1), self.info.output.height.max(1));

    if self
      .state
      .temporal
      .as_ref()
      .is_some_and(|history| !history.is_sized(width, height))
    {
      self.state.temporal = None;
    }

    let history: &TemporalHistory = self
      .state
      .temporal
      .get_or_insert_with(|| TemporalHistory::new(device, width, height));
    let current: Mat4 = view.get_view_projection();
    let (previous, previous_view): (Mat4, Mat4) = self.state.temporal_previous.unwrap_or((current, view.view));

    self.info.temporal = TemporalUniform::new(
      current,
      previous,
      previous_view,
      self.info.jitter,
      history.is_valid && self.state.temporal_previous.is_some(),
    );
    self.state.temporal_previous = Some((current, view.view));
  }

  /// Makes FSR 2's targets while it resolves, dropping them otherwise, and writes its constants.
  fn prepare_fsr(&mut self, device: &wgpu::Device, view: &CameraView) {
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
      .is_some_and(|fsr| !fsr.is_sized(render, display))
    {
      self.state.fsr = None;
    }

    let fsr: &FsrTargets = self
      .state
      .fsr
      .get_or_insert_with(|| FsrTargets::new(device, render, display, ViewTargets::SCENE));

    self.info.fsr = FsrUniform::new(
      (render, display),
      self.info.jitter,
      view,
      (FsrTargets::get_luma_mip_size(render), self.info.jitter_phases),
      fsr.frame_index,
    );
  }

  /// Lists the models' composited clusters in view, their places back to front by distance and each place's clusters in
  /// their parts' order, as the engine draws its sorted blended objects; binds the list for the composited pass.
  fn prepare_sorted(&mut self, scene: &LevelScene, device: &wgpu::Device, queue: &wgpu::Queue, view: &CameraView) {
    let planes: [Vec4; 6] = view.get_planes();
    let mut places: Vec<(f32, StaticSortedPlace<'_>)> = scene
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

  /// Notes the smoothing pass while one smooths the frame as drawn.
  fn prepare_smoothing(&mut self, passes: LevelPasses<'_>, mode: RenderAntialiasing) {
    // SMAA whose lookup textures could not be read smooths as FXAA does, which needs none.
    let mode: RenderAntialiasing = match (mode, passes.smaa) {
      (RenderAntialiasing::Smaa, None) => RenderAntialiasing::Fxaa,
      _ => mode,
    };
    let is_smoothed: bool = matches!(mode, RenderAntialiasing::Fxaa | RenderAntialiasing::Smaa)
      && self.state.targets.is_some()
      && self.info.debug_view == RenderDebugView::Final;

    self.info.smoothing = is_smoothed.then_some(mode);
  }

  /// Makes the frame at the viewport's size while the scene is drawn smaller, dropping it otherwise, and notes what the
  /// upscale passes read.
  fn prepare_upscale(&mut self, device: &wgpu::Device) {
    let Some(targets) = &self.state.targets else {
      return;
    };
    let output: RenderRect = self.info.output;

    if targets.is_sized(output.width, output.height) {
      self.state.upscale = None;
    } else if self
      .state
      .upscale
      .as_ref()
      .is_none_or(|upscale| !upscale.is_sized(output.width, output.height))
    {
      self.state.upscale = Some(UpscaleTargets::new(device, output.width, output.height));
    }

    self.info.upscale = UpscaleUniform {
      output_size: Vec2::new(output.width as f32, output.height as f32),
      sharpness: self.info.upscaling.get_sharpness(),
      _pad: 0.0,
    };
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
      self.info.smoothing.unwrap_or(RenderAntialiasing::None)
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
      indirect_light: self
        .info
        .bitmask
        .filter(|search| search.is_lit)
        .map(|search| RenderAppliedIndirectLight {
          intensity: self.info.indirect_light.intensity,
          is_shared: search.is_occluding,
        }),
      reflections: self.info.reflection.map(|trace| RenderAppliedReflections {
        intensity: trace.intensity,
        quality: trace.quality,
      }),
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
