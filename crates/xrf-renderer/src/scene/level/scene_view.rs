use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, Vec2, Vec3, Vec4};
use xrf_error::XrfResult;
use xrf_renderer_core::{FrameGraph, GraphBindings, GraphRuntime, GraphTexture, UniformBinding};

use crate::contract::render_ambient_occlusion_method::RenderAmbientOcclusionMethod;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_applied_indirect_light::RenderAppliedIndirectLight;
use crate::contract::render_applied_reflections::RenderAppliedReflections;
use crate::contract::render_applied_report::RenderAppliedReport;
use crate::contract::render_applied_shadows::RenderAppliedShadows;
use crate::contract::render_bloom_mode::RenderBloomMode;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_grass_report::RenderGrassReport;
use crate::contract::render_level_hit::RenderLevelHit;
use crate::contract::render_lights_report::RenderLightsReport;
use crate::contract::render_meter_reading::RenderMeterReading;
use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_particles_report::RenderParticlesReport;
use crate::contract::render_pool_use::RenderPoolUse;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_selection::RenderSelection;
use crate::contract::render_static_report::RenderStaticReport;
use crate::contract::render_trace_size::RenderTraceSize;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_water_settings::RenderWaterSettings;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::texture_copy::add_texture_copy;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_target_uses::ViewTargetUses;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::lighting::light_units::LightUnits;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::contact_shadow_uniform::ContactShadowUniform;
use crate::pass::grass_occlusion_parameters::GrassOcclusionParameters;
use crate::pass::level_passes::LevelPasses;
use crate::pass::light_trace_uniform::LightTraceUniform;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::rain_uniform::RainUniform;
use crate::pass::reflection_trace::ReflectionTrace;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::view_binding::ViewBinding;
use crate::pass::wet_uniform::WetUniform;
use crate::scene::level::cover_textures::CoverTextures;
use crate::scene::level::feature::bitmask_tracing::BitmaskTracing;
use crate::scene::level::feature::composited_draw::CompositedDraw;
use crate::scene::level::feature::fog_feature::FogFeature;
use crate::scene::level::feature::light_buffers::LightBuffers;
use crate::scene::level::feature::lights_feature::LightsFeature;
use crate::scene::level::feature::overhead_windows::OverheadWindows;
use crate::scene::level::feature::present_inputs::PresentInputs;
use crate::scene::level::feature::ray_guides::RayGuides;
use crate::scene::level::feature::resolve_feature::ResolveFeature;
use crate::scene::level::feature::scene_features::SceneFeatures;
use crate::scene::level::feature::sun_trace_feature::SunTraceFeature;
use crate::scene::level::feature::view_frame_end::ViewFrameEnd;
use crate::scene::level::feature::water_frame::WaterFrame;
use crate::scene::level::grass_card_handles::GrassCardHandles;
use crate::scene::level::grass_level::GrassLevel;
use crate::scene::level::grass_planting::GrassPlanting;
use crate::scene::level::grass_shadow_casters::GrassShadowCasters;
use crate::scene::level::grass_view_frame::GrassViewFrame;
use crate::scene::level::level_frame::LevelFrame;
use crate::scene::level::level_overlays::LevelOverlays;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::level_shadows::LevelShadows;
use crate::scene::level::level_surface::LevelSurface;
use crate::scene::level::light_shadow_atlas::LightShadowAtlas;
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::level::lighting_recorded::LightingRecorded;
use crate::scene::level::lights_frame::LightsFrame;
use crate::scene::level::rain_cover::RainCover;
use crate::scene::level::scene_output::SceneOutput;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::shadow_sway::ShadowSway;
use crate::scene::level::shadows_recorded::ShadowsRecorded;
use crate::scene::level::sky_views::SkyViews;
use crate::scene::level::surroundings_cube::SurroundingsCube;
use crate::scene::level::view_frame::ViewFrame;
use crate::scene::level::view_prepare::ViewPrepare;
use crate::scene::level::view_record::ViewRecord;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::static_scene::static_selection::StaticSelection;
use crate::scene::static_scene::static_sight::StaticSight;
use crate::scene::static_scene::static_slot_info::StaticSlotInfo;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// What a pick's texel says it met: a cluster, by its index and place, or an impostor, by its index.
const PICKED_CLUSTER: u32 = 1;
const PICKED_IMPOSTOR: u32 = 2;

/// A scene as one viewport draws it: the values its units share this frame (`ViewFrame`), its targets, and its units
/// (`SceneFeatures`), which each keep what they keep between frames. The scene (`LevelScene`) is shared by every view
/// of it and passed in; this prepares the view's frame from it and records it into the frame's graph.
pub struct SceneView {
  /// The values every unit reads this frame.
  shared: ViewFrame,
  /// The G-buffer, depth, light and scene it draws into, once its size is known.
  targets: Option<ViewTargets>,
  /// Its feature units.
  features: SceneFeatures,
}

impl SceneView {
  /// What the grass came to on a recent frame.
  pub fn take_grass_report(&mut self) -> RenderGrassReport {
    self.features.grass.take_report()
  }

  /// What the spot meter last read; none while the view does not meter.
  pub fn take_meter_reading(&mut self) -> Option<RenderMeterReading> {
    self.features.meter.take_reading()
  }

  /// The particles' report for this view: the scene's simulation since the last report and the quads this view drew.
  pub fn take_particles_report(&self, scene: &mut LevelScene) -> RenderParticlesReport {
    scene.particles.take_report(self.features.particles.get_drawn())
  }

  /// The size its scene is rendered at, once its targets are made.
  pub fn get_render_size(&self) -> Option<(u32, u32)> {
    self.targets.as_ref().map(|it| (it.width, it.height))
  }

  /// What smooths its frame as drawn: the temporal resolve, else the smoothing, none where neither runs.
  pub fn get_antialiasing(&self) -> RenderAntialiasing {
    self
      .features
      .resolve
      .get_temporal()
      .or(self.features.smoothing.get_mode())
      .unwrap_or(RenderAntialiasing::None)
  }

  /// Whether DLSS reconstructs rays as it resolves its frame.
  pub fn is_ray_reconstructed(&self) -> bool {
    self.features.resolve.is_ray_reconstructed()
  }

  /// What its window's present pass reads this frame.
  pub fn get_present_uniform(&self) -> &PresentUniform {
    self.features.present.get_uniform()
  }

  /// The size its frame is resolved at, none before its targets are made.
  pub fn get_output_size(&self) -> Option<(u32, u32)> {
    self.targets.as_ref().map(|_| self.features.resolve.get_resolved())
  }

  /// This frame's unjittered view projection and the last one's, which the surfaces' motion is measured between.
  pub fn next_motion(&mut self, current: Mat4) -> (Mat4, Mat4) {
    self.features.resolve.next_motion(current)
  }

  /// The last frame's jittered view projection, this frame's taking its place.
  pub fn next_jittered(&mut self, current: Mat4) -> Mat4 {
    self.features.resolve.next_jittered(current)
  }

  /// A view of a scene, nothing of it drawn yet.
  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, view_layout: &wgpu::BindGroupLayout) -> Self {
    Self {
      shared: ViewFrame::default(),
      targets: None,
      features: SceneFeatures::new((device, queue), view_layout),
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

  /// Sizes the targets to the viewport and readies every unit's frame, in the order each reads what an earlier one
  /// decided: the weather and the sky, the scene, the view's own units, then the lighting.
  pub fn prepare(&mut self, scene: &mut LevelScene, encoder: &mut FrameEncoder, frame: &ViewPrepare<'_>) {
    profiling::scope!("prepare view");

    let (width, height) = frame.drawn;

    if !self.targets.as_ref().is_some_and(|it| it.is_sized(width, height)) {
      self.targets = Some(ViewTargets::new(frame.device, width, height));
      self.features.statics.fit(frame.device, (width, height));
    }

    let time: f32 = scene.started.elapsed().as_secs_f32();

    // What of the scene drawn by passes of its own the view may see: none of the water's or the surface passes' work
    // runs for what is out of sight.
    let sight: StaticSight = scene.statics.get_sight(&frame.view.get_frustum());
    let lighting_frame: LightingFrame = self.prepare_weather(scene, frame, &sight, time);

    self
      .features
      .overlays
      .follow_skeletons((scene, frame.device, &mut *encoder));

    self.prepare_scene(scene, encoder, frame, &lighting_frame, time);
    self.prepare_view(scene, encoder, frame, &sight);
    self.prepare_lighting(scene, encoder, frame, &lighting_frame, time);
  }

  /// The units the weather drives: the shadows' maps, the exposure, the flares and the sky, the water, the wind, the
  /// falling rain and the wet surfaces following it, the thunder, and the post units reading the weather. Answers what
  /// the lighting draws the sky with.
  fn prepare_weather(
    &mut self,
    scene: &LevelScene,
    frame: &ViewPrepare<'_>,
    sight: &StaticSight,
    time: f32,
  ) -> LightingFrame {
    let (device, options, lighting) = (frame.device, frame.options, frame.lighting);
    let weather = (lighting, frame.weather);
    let size: Option<(u32, u32)> = self.targets.as_ref().map(|targets| (targets.width, targets.height));

    self.features.shadows.prepare(
      device,
      options.features.shadows.resolution,
      options.features.shadows.get_cascade_count() > 0,
    );
    self.features.exposure.prepare(frame.queue, options, Instant::now());

    let sun_sprite: Option<(String, Vec4)> = self.features.flares.prepare(
      device,
      weather,
      options,
      frame.weather_textures,
      (frame.view, frame.weather_rate),
      self.targets.is_some(),
    );

    self.features.sky.prepare(
      device,
      (frame.passes, options),
      (&lighting.sky, sun_sprite, &scene.environments.1),
      frame.weather_textures,
    );
    self.features.water.prepare(
      device,
      options,
      WaterFrame {
        targets: self.targets.as_ref(),
        is_in_view: sight.is_water(),
        intensity: lighting.water_intensity,
        wind: lighting.wind,
        rain: lighting.rain.map_or(0.0, |rain| rain.density),
        time,
      },
    );
    self.features.wind.prepare(options, lighting, time);

    // The falling rain while the weather rains and the view shows it, then the wet surfaces, which follow it; the
    // ambient effects blow the wind the rain reads this frame.
    let windows: OverheadWindows = self.get_overhead_windows();
    let falling: Option<RainUniform> = self.features.rain.prepare(
      device,
      weather,
      (
        (lighting.rain.filter(|_| options.mode.is_lit), &options.features.rain),
        scene.gust,
        (time, frame.view.get_look()),
      ),
      (windows.cover, windows.cover_shape),
      frame.weather_textures,
    );

    self.features.wet.prepare(
      weather,
      options,
      (&scene.wetness, falling.as_ref(), time),
      (windows, self.features.resolve.get_noise_frame()),
      frame.weather_textures,
    );
    self.follow_rain_surfaces(windows);

    // The enhanced drops' rings, where they land on the level water's map, which the water and the puddles ripple by.
    let rings: (f32, f32, f32) = (
      self.features.rain.get_rings(),
      options.features.rain.ring_distance.max(0.0),
      options.features.rain.ring_strength.max(0.0),
    );

    self.features.water.follow_rings(rings);
    self.features.wet.follow_rings(rings);
    self
      .features
      .thunder
      .prepare(device, weather, options, frame.weather_textures);

    let lighting_frame: LightingFrame = self.features.sky.get_lighting_frame(
      (&lighting.sky, options),
      frame.weather_textures,
      (
        self.features.exposure.is_adapting(),
        (
          self.features.exposure.get_pre_exposure(),
          LightUnits::of(options, lighting),
        ),
        time,
      ),
    );

    self.features.sun_shafts.prepare(options, lighting);
    self
      .features
      .bloom
      .prepare(frame.passes, options, lighting.engine, size);
    self.features.fog.prepare(options, lighting, size, time);

    lighting_frame
  }

  /// The scene's units: the static cull; the grass, which reads its discard, plants around last frame's puddles and
  /// casts by the trace or the cascades; the lights, which read the cull's levels of detail; then what the views off
  /// the viewport draw the scene with.
  fn prepare_scene(
    &mut self,
    scene: &mut LevelScene,
    encoder: &mut FrameEncoder,
    frame: &ViewPrepare<'_>,
    lighting_frame: &LightingFrame,
    time: f32,
  ) {
    let (device, options, lighting, view) = (frame.device, frame.options, frame.lighting, frame.view);

    self
      .features
      .statics
      .prepare(&scene.statics, options, (view, frame.field_of_view), frame.drawn);

    let is_traced: bool = options.is_light_traced() && frame.passes.scene_trace.is_traced() && scene.trace.is_some();
    let is_cascaded: bool =
      SunTraceFeature::is_cascaded(options, is_traced, self.features.resolve.is_ray_reconstructed());

    let grass: GrassViewFrame<'_> = GrassViewFrame {
      camera: view,
      discard_below: self.features.statics.get_cull().discard_below,
      // The ambient effects blow the wind the grass reads this frame.
      time,
      is_windy: lighting.trees.is_some(),
      strength: scene.gust.strength,
      foliage: self.features.wind.get_values(),
      // Where the puddles stood last frame, which no tuft is planted in: the wet readies after the grass.
      puddles: LevelSurface::get_grass_puddles(
        self.features.level_surface.as_ref(),
        frame.passes.wet.get_dry()[1],
        self.features.wet.has_puddles(),
      ),
      is_traced,
      cascade_reach: is_cascaded
        .then(|| LevelShadows::get_grass_reach(&options.features.shadows, options.features.grass.shadow_cascades))
        .flatten(),
    };

    self
      .features
      .grass
      .prepare(device, scene.grass.get_level(), &options.features.grass, &grass);
    scene.lights.prepare(
      &mut *encoder,
      &mut self.features.lights,
      LightsFrame {
        camera: view,
        settings: &options.features.lights,
        engine: lighting.engine,
        lod: (
          self.features.statics.get_cull().glod_start,
          self.features.statics.get_cull().glod_end,
        ),
        statics: &scene.statics,
        sway: &Self::to_sway(&scene.statics, self.features.wind.get_sway()),
        grass_reach: self.features.grass.get_cascade_reach(),
        campfire_shares: &scene.campfire_shares,
        is_scene_first: frame.is_scene_first,
        is_physical: options.is_light_physical(),
        light_scale: lighting_frame.light_units.local * lighting_frame.pre_exposure.current,
      },
    );
    self.shared.camera = *view;
    self.shared.sun_direction = lighting.get_sun_direction();
    // A view off the viewport draws the static scene textured, bumped as the view is, by the hemisphere's strength, and
    // never with parallax.
    self.shared.switches = Vec4::new(
      1.0,
      options.mode.is_bumped as u32 as f32,
      options.features.hemi_strength,
      0.0,
    );
    self.shared.shadow_settings = options.features.shadows.clone();
    self.shared.is_wireframe = options.mode.is_wireframe;
  }

  /// The view's own units: the resolve and the smoothing, the screen's occlusion and the bitmask search at half the
  /// frame's size, the reflections, the view's trace, the composited surfaces in `sight`, and the spot meter and the
  /// calibration props.
  fn prepare_view(
    &mut self,
    scene: &LevelScene,
    encoder: &mut FrameEncoder,
    frame: &ViewPrepare<'_>,
    sight: &StaticSight,
  ) {
    let (device, options, view) = (frame.device, frame.options, frame.view);
    let (width, height) = frame.drawn;
    let half: (u32, u32) = (width.div_ceil(2), height.div_ceil(2));

    self.features.resolve.prepare(
      (device, frame.dlss),
      view,
      (
        self.targets.as_ref().map(|targets| (targets.width, targets.height)),
        frame.resolved,
        frame.output,
      ),
      (options.output.upscaling, options.is_scene_linear()),
    );
    self.features.smoothing.prepare(
      frame.passes,
      options.features.antialiasing,
      (
        self.targets.is_some() && options.mode.debug_view == RenderDebugView::Final,
        options.is_scene_linear(),
      ),
    );
    self.features.ambient_occlusion.prepare(options, view.projection, half);
    self.features.bitmask.prepare(device, options, view.projection, half);
    self.features.indirect_trace.prepare(
      frame.passes.indirect_trace.is_traced()
        && self.features.resolve.is_ray_reconstructed()
        && self.features.bitmask.get_search().is_some_and(|search| search.is_lit),
      (
        self.features.bitmask.get_indirect_light().intensity,
        self.features.resolve.get_noise_frame(),
      ),
    );
    self.features.reflections.prepare(
      device,
      options,
      frame.drawn,
      (self.features.resolve.is_ray_reconstructed(), self.features.wet.is_wet()),
    );
    self.features.view_trace.prepare(
      (device, &mut *encoder),
      (scene.trace.as_ref(), self.features.grass.get_cards_blas()),
      view.position,
      options.is_scene_traced() && frame.passes.scene_trace.is_traced(),
    );
    self
      .features
      .composited
      .prepare((scene, device, &mut *encoder), view, options, sight);
    self.features.meter.prepare(device, options);
    self.features.calibration.prepare(options, view);
  }

  /// The lighting the passes read, the enhanced fog's height fog faded out indoors; the units reading it (the combine,
  /// the contact shadows and the sun's trace); then the particles, the present and the volumetric light.
  fn prepare_lighting(
    &mut self,
    scene: &mut LevelScene,
    encoder: &mut FrameEncoder,
    frame: &ViewPrepare<'_>,
    lighting_frame: &LightingFrame,
    time: f32,
  ) {
    let (device, options, view) = (frame.device, frame.options, frame.view);
    let noise_frame: u32 = self.features.resolve.get_noise_frame();

    self.shared.lighting = LightingUniform::new(frame.lighting, view.view, options, lighting_frame);
    self.shared.lighting.height_fog.w *= FogFeature::get_indoor_share(scene.camera_hemi);
    self
      .features
      .combine
      .prepare(&self.shared.lighting, view.get_depth_range().1);
    self.features.contact_shadows.prepare(
      options,
      (self.shared.lighting.to_sun, self.shared.lighting.sun),
      (frame.drawn.1, noise_frame),
    );
    self.features.sun_trace.prepare(
      options,
      self.shared.lighting.to_sun,
      noise_frame,
      self.features.resolve.is_ray_reconstructed(),
    );

    scene.particles.fill(view, options, &mut self.features.particles);

    self.features.particles.upload((device, &mut *encoder));
    self.prepare_present(options, frame.output, time);
    self.features.volumetric.prepare(device, options);
  }

  /// Writes where the rain cover and the level's surface now stand into what reads them, once `commit_periodic` has
  /// moved them: written before, a frame redrawing one would read it through where it stood the frame before.
  fn follow_overhead_maps(&mut self) {
    let windows: OverheadWindows = self.get_overhead_windows();

    self.features.rain.follow_cover(windows.cover, windows.cover_shape);
    self.features.wet.follow(windows);
    self.follow_rain_surfaces(windows);
  }

  /// Follows the rain to the level water's and surface's maps, which the enhanced drops land on while they are drawn.
  fn follow_rain_surfaces(&mut self, windows: OverheadWindows) {
    let is_drawn: bool = self.features.level_surface.is_some() && self.features.wet.is_enhanced();

    self
      .features
      .rain
      .follow_surfaces((windows.water, windows.surface, windows.surface_shape), is_drawn);
  }

  /// Where the rain cover and the level's surface stand now.
  fn get_overhead_windows(&self) -> OverheadWindows {
    let (cover, cover_shape): (Vec4, (f32, f32)) = self
      .features
      .rain_cover
      .as_ref()
      .map_or(RainCover::get_unmade(), |cover| (cover.get_window(), cover.get_shape()));
    let ((surface, surface_shape), water): ((Vec4, Vec4), Vec4) = self
      .features
      .level_surface
      .as_ref()
      .map_or(LevelSurface::get_unmade(), |surface| {
        (surface.get_window(), surface.get_water_window())
      });

    OverheadWindows {
      cover,
      cover_shape,
      surface,
      surface_shape,
      water,
    }
  }

  /// Offers `scheduler` what the view draws once in so many frames and wants this frame: the sun's cascades, the
  /// surroundings' face, the rain cover, the level's surface and the sky's refill. `commit_periodic` then readies what
  /// was granted.
  pub fn offer_periodic(
    &mut self,
    scene: &LevelScene,
    device: &wgpu::Device,
    scheduler: &mut PeriodicScheduler,
    view_layout: &wgpu::BindGroupLayout,
  ) {
    let SceneView { shared, features, .. } = self;
    let SceneFeatures {
      shadows,
      rain_cover,
      level_surface,
      overhead_layout: overhead_view_layout,
      surroundings,
      reflections,
      rain,
      wet,
      sky,
      ..
    } = features;
    let statics: &StaticScene = &scene.statics;
    let frame: ShadowFrame<'_> = ShadowFrame {
      scene: statics,
      camera: &shared.camera,
      settings: &shared.shadow_settings,
      sun_direction: shared.sun_direction,
      is_sunlit: shared.lighting.sun.truncate() != Vec3::ZERO,
      switches: shared.switches,
      sway: Self::to_sway(statics, features.wind.get_sway()),
      grass_reach: features.grass.get_cascade_reach(),
    };

    shadows.prepare_cascades(device, view_layout, &frame, scheduler);

    // The cube is for the traced reflections alone; the engine's reflect its own.
    if reflections.get_settings().is_drawn() {
      surroundings.offer(device, &frame, reflections.get_settings().probe, scheduler);
    } else {
      surroundings.rest();
    }

    // Out to the wet reach where the wetting reaches past the engine's.
    if rain.is_raining() || wet.is_wet() {
      rain_cover
        .get_or_insert_with(|| RainCover::new(device, overhead_view_layout))
        .offer(&frame, wet.get_reach(), scheduler);
    } else {
      *rain_cover = None;
    }

    // The puddles seat on it.
    if wet.is_enhanced() {
      level_surface
        .get_or_insert_with(|| LevelSurface::new(device, overhead_view_layout))
        .offer(&frame, scene.lights.get_fires(), scheduler);
    } else {
      *level_surface = None;
    }

    sky.offer_refill(scheduler);
  }

  /// Readies what `scheduler` granted of what `offer_periodic` offered, which the frame's passes then draw, and writes
  /// where the overhead maps now stand; `encoder` takes what readying them writes.
  pub fn commit_periodic(
    &mut self,
    scene: &LevelScene,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    scheduler: &PeriodicScheduler,
  ) {
    let SceneView { shared, features, .. } = self;
    let SceneFeatures {
      shadows,
      rain_cover,
      level_surface,
      surroundings,
      reflections,
      sky,
      ..
    } = features;
    let statics: &StaticScene = &scene.statics;
    let frame: ShadowFrame<'_> = ShadowFrame {
      scene: statics,
      camera: &shared.camera,
      settings: &shared.shadow_settings,
      sun_direction: shared.sun_direction,
      is_sunlit: shared.lighting.sun.truncate() != Vec3::ZERO,
      switches: shared.switches,
      sway: Self::to_sway(statics, features.wind.get_sway()),
      grass_reach: features.grass.get_cascade_reach(),
    };

    shadows.commit_cascades(device, encoder, &frame, scheduler);

    if reflections.get_settings().is_drawn() {
      surroundings.commit(device, encoder, &frame, reflections.get_settings().probe, scheduler);
    }

    if let Some(rain_cover) = rain_cover {
      rain_cover.commit(device, encoder, &frame, scheduler);
    }

    if let Some(level_surface) = level_surface {
      level_surface.commit(device, encoder, &frame, scheduler);
    }

    sky.commit_refill(scheduler);
    self.follow_overhead_maps();
  }

  /// Readies the scene's light shadow faces `scheduler` granted of those every view of it asked for this frame, from
  /// this view's culling; `encoder` takes the copies a list growing makes.
  pub fn commit_scene_shadows(
    &self,
    scene: &mut LevelScene,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    scheduler: &PeriodicScheduler,
  ) {
    let LevelScene { statics, lights, .. } = scene;
    let frame: ShadowFrame<'_> = ShadowFrame {
      scene: statics,
      camera: &self.shared.camera,
      settings: &self.shared.shadow_settings,
      sun_direction: self.shared.sun_direction,
      is_sunlit: self.shared.lighting.sun.truncate() != Vec3::ZERO,
      switches: self.shared.switches,
      sway: Self::to_sway(statics, self.features.wind.get_sway()),
      grass_reach: self.features.grass.get_cascade_reach(),
    };

    lights.commit_shadows(device, encoder, &frame, scheduler);
  }

  /// Declares the lighting: the lights binned into the view's clusters, the contact shadows marched, the sun then every
  /// light drawn into the light target, the binning's overflow read back, and the ambient occlusion and the indirect
  /// light searched and filtered. Answers the indirect light filtered, none where it is not gathered, and how much of
  /// the sun reaches each pixel.
  fn add_lighting_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    (view, textures, scene): (&'a ViewBinding, &'a wgpu::BindGroup, &'a LevelScene),
    (targets, lit, traced): (
      ViewTargetHandles,
      LightingHandles<'a>,
      Option<(TraceParameters<'a>, &'a wgpu::BindGroup)>,
    ),
  ) -> (Option<GraphTexture>, GraphTexture) {
    let lights: &'a LightsFeature = &self.features.lights;
    let buffers: LightBuffers = lights.add_binning_passes((&mut *graph, &mut *bindings, &mut *runtime), passes);

    // The contact shadows the sun multiplies its own by, and the uniform the lights march theirs by.
    let (contact_shadows, contact): (GraphTexture, UniformBinding<ContactShadowUniform>) = self
      .features
      .contact_shadows
      .add_passes((&mut *graph, &mut *bindings, &mut *runtime), passes, view, targets);
    // What the sun sees, traced where DLSS ray reconstruction denoises it; the cascades shadow it otherwise.
    let sun_traced: Option<GraphTexture> =
      self
        .features
        .sun_trace
        .add_passes((&mut *graph, &mut *runtime), passes, view, targets, traced);
    let sun_seen: GraphTexture = self.features.sun.add_passes(
      (&mut *graph, &mut *bindings),
      passes,
      view,
      (targets, lit),
      (
        contact_shadows,
        sun_traced,
        self.features.shadows.get_values().filter_reach,
      ),
    );

    // The shadowed lights traced where DLSS ray reconstruction denoises them, their atlas faces otherwise.
    let traced_lights: Option<(
      TraceParameters<'a>,
      &'a wgpu::BindGroup,
      UniformBinding<LightTraceUniform>,
    )> = traced
      .filter(|_| self.features.resolve.is_ray_reconstructed())
      .map(|(trace, textures)| {
        let light_trace: LightTraceUniform = LightTraceUniform::new(self.features.resolve.get_noise_frame());

        (trace, textures, runtime.push_uniform(&light_trace))
      });

    lights.add_draw_passes(
      (&mut *graph, &mut *bindings),
      passes,
      (view, textures, scene.lights.get_shadow_atlas()),
      (targets, lit, buffers),
      (contact, self.features.contact_shadows.get_light_count()),
      traced_lights,
    );
    lights.add_overflow_pass(&mut *graph, buffers);
    // The occlusion and the indirect light recorded beside the lights, so neither group bounds the frame's encoding.
    graph.begin_group("occlusion");

    // GTAO searches on its own; VBAO, the indirect light or both share one bitmask search.
    self
      .features
      .ambient_occlusion
      .add_passes((&mut *graph, &mut *bindings, &mut *runtime), passes, view, targets);

    let indirect_light: Option<GraphTexture> = self.features.bitmask.add_passes(
      (graph, bindings, runtime),
      passes,
      view,
      targets,
      BitmaskTracing {
        traced,
        is_light_traced: self.features.indirect_trace.is_traced(traced.is_some()),
      },
    );

    (indirect_light, sun_seen)
  }

  /// Ends a frame its graph recorded: every unit's history it wrote becomes the one the next frame keeps.
  pub fn end_frame(&mut self) {
    let mut end: ViewFrameEnd<'_> = ViewFrameEnd {
      depth: self.targets.as_mut().map(|targets| &mut targets.depth),
    };

    for feature in self.features.list_mut() {
      feature.end_frame(&mut end);
    }
  }

  /// Notes what the window pass reads, once every other unit knows what it draws: whether the water, a particle, the
  /// enhanced rain's splashes or a distorting surface of the scene moves what is seen through it, which bloom it lays
  /// over the frame, and the sky debanding, its noise turning with `time`.
  fn prepare_present(&mut self, options: &RenderViewOptions, output: RenderRect, time: f32) {
    if self.targets.is_none() {
      return;
    }

    let water: &RenderWaterSettings = &options.features.water;
    let is_water_distorting: bool = water.is_enabled && water.is_distorted && options.mode.is_lit;
    let features: &SceneFeatures = &self.features;
    let inputs: PresentInputs = PresentInputs {
      is_occluded: features.ambient_occlusion.is_drawn(),
      is_indirect: features.bitmask.get_search().is_some_and(|search| search.is_lit),
      reflection_ratio: features.reflections.get_trace().map(ReflectionTrace::get_ratio),
      is_distorting: !options.mode.is_wireframe
        && (is_water_distorting
          || features.particles.is_distorting()
          || features.rain.is_distorting()
          || features.composited.is_distorting()),
      selection_color: features.overlays.get_selection_color(),
      is_engine_bloomed: features.bloom.is_engine_drawn(),
      is_enhanced_bloomed: features.bloom.is_enhanced_drawn(),
      grass_reach: features.grass.get_cascade_reach(),
    };

    self
      .features
      .present
      .prepare(options, (self.features.resolve.is_rescaled(), output), inputs, time);
  }

  /// Readies a frame for its graph, none before the view's targets are made: a pick's camera narrowed to the texel
  /// picked and the readback it is copied into while one is free, and the depth the next frame's occlusion tests.
  pub fn begin_frame(
    &mut self,
    (device, encoder): (&wgpu::Device, &mut FrameEncoder),
    pick: Option<(&CameraUniform, &wgpu::BindGroupLayout, bool)>,
  ) -> Option<LevelFrame> {
    if self.targets.is_none() || !self.features.statics.is_ready() {
      return None;
    }

    let pick_slot: Option<usize> =
      pick.and_then(|(camera, view_layout, _)| self.features.pick.ready((device, encoder), camera, view_layout));

    self.features.statics.begin_frame();

    Some(LevelFrame {
      pick_slot,
      is_pick_through: pick.is_some_and(|(_, _, is_through)| is_through),
      is_scene_first: false,
    })
  }

  /// Declares the frame `begin_frame` readied into the frame's graph, in the encode groups the frame is recorded in, on
  /// as many threads where it is not timed: the scene (culled and drawn into the G-buffer, the grass planted and
  /// drawn), its shadows (the cascades, the rain's cover, the level's surface and the lamps' faces), its lighting with
  /// the reflections and the water, and what blends over it and resolves it; the pick last. Which passes run is decided
  /// by the units; each reads the view, which recording leaves as it is. The present pass reads what it leaves.
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

    let targets: &'a ViewTargets = self.targets.as_ref()?;
    let uses: ViewTargetUses = ViewTargetUses {
      is_occluded: self.features.ambient_occlusion.is_drawn(),
      is_hazed: self.features.sky.is_hazing(),
      is_engine_bloomed: self.features.bloom.is_engine_drawn(),
    };
    let handles: ViewTargetHandles =
      ViewTargetHandles::import(&mut *graph, &mut *bindings, (targets, uses, passes.vbao.get_dark()));
    let statics: StaticSceneHandles = self.features.statics.import(
      (&mut *graph, &mut *bindings, runtime),
      &scene.statics,
      self.features.wind.get_uniform(),
    )?;
    let record: ViewRecord<'a> = ViewRecord {
      passes,
      view,
      textures: textures.get_bind_group(),
      handles,
      statics,
      cull: self.features.statics.get_cull(),
      lit: self.import_lighting((&mut *graph, &mut *bindings, runtime), passes),
      unlit: bindings.import_view(&mut *graph, "indirect light none", passes.vbao.get_dark()),
      untraced: bindings.import_view(&mut *graph, "reflections none", passes.reflections.get_untraced()),
      no_ends: bindings.import_view(&mut *graph, "reflection ends none", passes.reflections.get_no_ends()),
    };

    graph.begin_group("scene");

    let grass_cards: Option<GrassCardHandles> =
      self.record_scene(scene, (&mut *graph, &mut *bindings, runtime), record);

    graph.begin_group("shadows");

    let shadowed: ShadowsRecorded = self.record_shadows(
      scene,
      (&mut *graph, &mut *bindings, runtime),
      record,
      (grass_cards, frame.is_scene_first),
    );

    graph.begin_group("lighting");

    // The rings the enhanced drops leave where they land, while the puddles or the enhanced water ripple by them.
    let rings: GraphTexture = self
      .features
      .rain
      .add_rings(
        (&mut *graph, &mut *bindings, runtime),
        passes,
        view,
        (
          shadowed.covers.map(|covers| covers.depth),
          shadowed.surface[2],
          shadowed.surface[0],
        ),
        self.features.wet.has_puddles() || self.features.water.is_enhanced(),
      )
      .unwrap_or_else(|| bindings.import_view(&mut *graph, "rain rings none", passes.rain.get_nothing()));
    let (wet_surface, wet_reach): (GraphTexture, GraphTexture) = self.features.wet.add_passes(
      (&mut *graph, &mut *bindings),
      passes,
      view,
      (handles, shadowed.covers, shadowed.wet, rings),
      shadowed.surface,
    );
    let mut output: SceneOutput = SceneOutput {
      targets: handles,
      scene: handles.scene,
      shown: handles.scene,
      present: runtime.push_uniform(self.features.present.get_uniform()),
      lighting: record.lit.lighting,
      indirect_light: record.unlit,
      reflections: record.untraced,
      reflection_ends: record.no_ends,
      bloom: handles.bloom[0],
    };

    let lit: LightingRecorded<'a> = self.record_lighting(
      scene,
      (&mut *graph, &mut *bindings, runtime),
      record,
      (grass_cards, wet_surface, wet_reach, rings),
    );

    graph.begin_group("post");

    self.record_post(
      scene,
      (&mut *graph, &mut *bindings, runtime),
      record,
      (lit, shadowed),
      &mut output,
    );

    if let Some(slot) = frame.pick_slot {
      self.features.pick.add_passes(
        (&mut *graph, &mut *bindings),
        passes.gbuffer,
        (record.textures, &statics),
        (slot, frame.is_pick_through),
      );
    }

    Some(output)
  }

  /// The lighting every lit pass binds: the view's lighting uniform, the exposure, the material table and the sun's
  /// shadow maps with what they were drawn with.
  fn import_lighting<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
  ) -> LightingHandles<'a> {
    LightingHandles {
      lighting: runtime.push_uniform(&self.shared.lighting),
      exposure: bindings.import_buffer(&mut *graph, "exposure", &self.features.exposure.state),
      material_lut: bindings.import_view(&mut *graph, "material table", &passes.table.view),
      lut_sampler: &passes.table.sampler,
      shadow_maps: bindings.import_view(
        &mut *graph,
        "sun shadow maps",
        self
          .features
          .shadows
          .get_maps()
          .map_or(passes.sun.get_unshadowed(), |maps| &maps.view),
      ),
      shadows: runtime.push_uniform(self.features.shadows.get_values()),
    }
  }

  /// The scene's group: the distortion cleared, the grass planted, the static scene culled and drawn into the G-buffer,
  /// the grass culled against its depth and its cards stood, the reflections' depth kept before or after the grass
  /// draws, the wall marks and the sky's motion. Answers the grass's cards.
  fn record_scene<'a>(
    &'a self,
    scene: &'a LevelScene,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    record: ViewRecord<'a>,
  ) -> Option<GrassCardHandles> {
    let ViewRecord {
      passes,
      view,
      textures,
      handles,
      statics,
      ..
    } = record;
    let is_drawn: bool = !self.shared.is_wireframe;
    let grass_level: Option<&GrassLevel> = scene.grass.get_level();

    ViewTargets::add_distortion_clear(&mut *graph, handles);

    let grass_planting: Option<GrassPlanting> = self.features.grass.add_planting(
      (&mut *graph, &mut *bindings, &mut *runtime),
      (passes.grass, grass_level),
    );

    self.features.statics.add_gbuffer_passes(
      (&mut *graph, &mut *bindings),
      passes,
      (view, textures, &statics),
      handles,
    );
    if is_drawn {
      self
        .features
        .calibration
        .add_passes((&mut *graph, &mut *runtime), passes, view, handles);
    }

    // The grass is culled once the static scene's depth is reduced, so a cell behind it plants nothing to draw.
    if let Some(grass_planting) = grass_planting {
      let occlusion: GrassOcclusionParameters = GrassOcclusionParameters {
        pyramid: statics.pyramid,
        occlusion: runtime.push_uniform(&self.features.statics.get_frame_occlusion()),
      };

      self
        .features
        .grass
        .add_cull(&mut *graph, (passes.grass, grass_level), (grass_planting, occlusion));
    }

    // The grass's cards, which cast its shadows into the sun's cascades or the trace, stood once its ring is planted.
    let grass_cards: Option<GrassCardHandles> = grass_planting.and_then(|grass_planting| {
      self.features.grass.add_cards(
        (&mut *graph, &mut *bindings),
        (passes.grass, grass_level),
        grass_planting,
      )
    });
    // The depth the reflections walk their rays over, kept before the grass draws, or after it where the grass is
    // reflected.
    let is_grass_reflected: bool = self.features.reflections.get_settings().is_grass_reflected;

    if !is_grass_reflected {
      self
        .features
        .reflections
        .add_pyramid_passes((&mut *graph, &mut *bindings), passes, handles.depth);
    }

    self.features.statics.add_stats_pass(&mut *graph, &statics);

    if let (true, Some(grass_planting)) = (is_drawn, grass_planting) {
      self.features.grass.add_draw(
        &mut *graph,
        (passes.grass, grass_level),
        (handles, grass_planting),
        (view, textures),
      );
    }

    if is_grass_reflected {
      self
        .features
        .reflections
        .add_pyramid_passes((&mut *graph, &mut *bindings), passes, handles.depth);
    }

    if is_drawn {
      self
        .features
        .statics
        .add_wallmark_passes(&mut *graph, passes, (view, textures, &statics), handles);
      self.features.sky.add_motion_pass(&mut *graph, passes, view, handles);
    }

    grass_cards
  }

  /// The shadows' group: the sun's cascades with the grass's cards, the rain's cover while it rains or the surfaces are
  /// wet, the level's surface the enhanced wetting places its puddles by, and the lamps' faces where this view draws
  /// the scene's first (`is_scene_first`).
  fn record_shadows<'a>(
    &'a self,
    scene: &'a LevelScene,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    record: ViewRecord<'a>,
    (grass_cards, is_scene_first): (Option<GrassCardHandles>, bool),
  ) -> ShadowsRecorded {
    let ViewRecord {
      passes,
      textures,
      statics,
      cull,
      lit,
      ..
    } = record;
    let grass_casters: Option<GrassShadowCasters> = self.features.grass.get_shadow_casters(
      (&mut *graph, &mut *bindings),
      (passes.grass, scene.grass.get_level(), textures),
      grass_cards,
    );

    self.features.shadows.add_passes(
      (&mut *graph, &mut *bindings),
      passes,
      (&statics, lit.shadow_maps),
      (cull, textures),
      grass_casters,
    );

    // The rain's cover, which its passes draw and reduce and the rain and the wet surfaces read.
    let covers: Option<CoverTextures> = self
      .features
      .rain_cover
      .as_ref()
      .filter(|_| self.features.rain.is_raining() || self.features.wet.is_wet())
      .map(|rain_cover| {
        rain_cover.add_passes(
          (&mut *graph, &mut *bindings, &mut *runtime),
          passes,
          &statics,
          (cull, textures),
        )
      });
    let wet: Option<UniformBinding<WetUniform>> = self.features.wet.push_uniform(runtime);
    // The level's surface seen from overhead, its lowest heights, its puddle sites and its water, which the enhanced
    // wetting places its puddles by.
    let surface: [GraphTexture; 3] = LevelSurface::add_surface_passes(
      self
        .features
        .level_surface
        .as_ref()
        .filter(|_| self.features.wet.is_enhanced()),
      (&mut *graph, &mut *bindings, runtime),
      passes,
      &statics,
      (cull, textures),
      (wet, self.features.water.get_uniform()),
    );

    if is_scene_first {
      scene.lights.add_shadow_passes(
        (&mut *graph, &mut *bindings),
        passes,
        &statics,
        (cull, textures),
        grass_casters,
      );
    }

    ShadowsRecorded { covers, wet, surface }
  }

  /// The lighting's group and the reflections': the scene's trace, the lights and the sun, the sky, the traced indirect
  /// light, the reflections and combine, the guides ray reconstruction reads, the frame copied before the water, and
  /// the water.
  fn record_lighting<'a>(
    &'a self,
    scene: &'a LevelScene,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    record: ViewRecord<'a>,
    (grass_cards, wet_surface, wet_reach, rings): (Option<GrassCardHandles>, GraphTexture, GraphTexture, GraphTexture),
  ) -> LightingRecorded<'a> {
    let ViewRecord {
      passes,
      view,
      textures,
      handles,
      statics,
      cull,
      lit,
      unlit,
      untraced,
      ..
    } = record;
    // The scene's trace, which RTAO, the reflections and the water trace their rays through.
    let traced: Option<(TraceParameters, &wgpu::BindGroup)> = self.features.view_trace.add_scene_trace(
      scene,
      (&mut *graph, &mut *bindings, runtime),
      record,
      (grass_cards, &self.features.lights, &self.features.grass),
    );
    let (mut indirect_light, sun_seen): (Option<GraphTexture>, GraphTexture) = self.add_lighting_passes(
      (&mut *graph, &mut *bindings, runtime),
      passes,
      (view, textures, scene),
      (handles, lit, traced),
    );

    self
      .features
      .statics
      .add_overdraw_pass(&mut *graph, passes, (view, textures, &statics), handles.light);
    let surroundings: &SurroundingsCube = &self.features.surroundings;
    let sky: Option<SkyParameters> = self.features.sky.add_passes(
      (&mut *graph, &mut *bindings, runtime),
      passes,
      (handles, lit),
      surroundings
        .is_active()
        .then_some((surroundings, &statics, (cull, textures))),
    );
    let mut reflections: Option<GraphTexture> = None;
    let mut reflection_ends: Option<GraphTexture> = None;
    let mut ray_guides: Option<RayGuides> = None;

    // The reflections and combine record in a group of their own, encoded beside the lights before them.
    graph.begin_group("reflections");

    // The last frame's scene and depth the reflections keep: read by this frame's hits, written again after the
    // particles draw.
    let source: Option<(GraphTexture, GraphTexture)> =
      self.features.reflections.import_source((&mut *graph, &mut *bindings));

    if let Some(sky) = sky {
      let mut traced_rays: Option<GraphTexture> = None;
      // How far the traced indirect light's rays went, where it is traced.
      let mut indirect_hits: Option<GraphTexture> = None;

      if let Some((light, hits)) =
        self
          .features
          .indirect_trace
          .add_passes((&mut *graph, &mut *runtime), passes, view, (handles, lit, sky), traced)
      {
        indirect_light = Some(light);
        indirect_hits = Some(hits);
      }

      if let Some((history, ends, traced)) = self.features.reflections.add_passes(
        (&mut *graph, &mut *bindings, runtime),
        passes,
        view,
        (handles, lit, wet_surface),
        source,
        (&sky, traced),
      ) {
        reflections = Some(history);
        reflection_ends = Some(ends);
        traced_rays = Some(traced);
      }

      self.features.combine.add_passes(
        (&mut *graph, &mut *bindings),
        passes,
        view,
        (handles, lit, sky),
        [
          indirect_light.unwrap_or(unlit),
          reflections.unwrap_or(untraced),
          wet_surface,
          wet_reach,
          sun_seen,
        ],
      );
      ray_guides = self.features.resolve.add_guide_passes(
        &mut *graph,
        passes,
        view,
        (handles, lit.lighting, wet_surface, traced_rays.unwrap_or(untraced)),
        (indirect_hits, unlit),
      );
    }

    let water_views: Option<&SkyViews> = self.features.sky.get_views();
    let before_water: Option<GraphTexture> = self.features.resolve.add_before_water(
      &mut *graph,
      handles,
      (
        ray_guides.is_some(),
        water_views.is_some() && self.features.water.is_drawn() && self.features.water.is_enhanced(),
      ),
    );
    let opaque: Option<GraphTexture> = self
      .features
      .resolve
      .add_reactive_passes((&mut *graph, &mut *bindings), (handles.scene, before_water));

    let depth_with_water: Option<GraphTexture> = self.features.water.add_passes(
      (&mut *graph, &mut *bindings, runtime),
      record,
      (
        water_views,
        before_water,
        self.features.statics.list_draw_args(&statics),
        rings,
        ray_guides,
      ),
      sky.zip(traced).map(|(sky, (trace, textures))| (sky, trace, textures)),
    );

    LightingRecorded {
      indirect_light,
      reflections,
      reflection_ends,
      sky,
      ray_guides,
      before_water,
      opaque,
      depth_with_water,
      source,
    }
  }

  /// The post group: the composited surfaces and the particles, the reflections' source for the next frame, the light
  /// volumes and the sun shafts, the rain and the thunder, the frame after its transparent layers for ray
  /// reconstruction, the flares, the fog, the blooms and the selection, the smoothing, the resolve and the exposure;
  /// the finished frame, the image shown and the enhanced bloom written into `output`.
  fn record_post<'a>(
    &'a self,
    scene: &'a LevelScene,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    record: ViewRecord<'a>,
    (lit_frame, shadowed): (LightingRecorded<'a>, ShadowsRecorded),
    output: &mut SceneOutput,
  ) {
    let ViewRecord {
      passes,
      view,
      textures,
      handles,
      statics,
      lit,
      ..
    } = record;
    let LightingRecorded {
      mut ray_guides,
      before_water,
      opaque,
      depth_with_water,
      source,
      sky,
      ..
    } = lit_frame;
    let is_drawn: bool = !self.shared.is_wireframe;
    let light_atlas: LightShadowAtlas<'a> = scene.lights.get_shadow_atlas();
    let composited_draw: Option<(CompositedDraw, SkyParameters)> = sky
      .filter(|_| is_drawn && self.features.sky.get_views().is_some())
      .map(|sky| {
        (
          CompositedDraw::new(record, self.features.statics.list_draw_args(&statics)),
          sky,
        )
      });

    if let Some((draw, sky)) = &composited_draw {
      self.features.composited.add_passes(&mut *graph, passes, (draw, *sky));
    }

    if is_drawn && self.features.particles.is_drawing() {
      self.features.particles.add_passes(
        (&mut *graph, &mut *bindings),
        passes.particles,
        (handles, lit.lighting, scene.particles.get_surfaces()),
        (view, textures),
      );
    }

    if let Some((draw, sky)) = &composited_draw {
      self
        .features
        .composited
        .add_after_particles((&mut *graph, &mut *bindings, runtime), passes, (draw, *sky, lit));
    }

    // What a reflection reads a frame later: the scene with the water and the particles drawn, before the sun shafts,
    // the rain and the flares, which belong to this camera and not to what it reflects. The depth it stood on is the
    // target itself, which the reflections keep at the frame's end.
    if let Some((source_scene, _)) = source {
      add_texture_copy(&mut *graph, "reflection source scene", handles.scene, source_scene);
    }

    // The volumetrics, the flares and the bloom, then the resolve, recorded beside the forward passes.
    graph.begin_group("effects");

    let (light_volumes, is_froxeled, froxels) = self.features.volumetric.add_passes(
      (&mut *graph, &mut *bindings, runtime),
      record,
      (&self.features.lights, light_atlas),
    );

    self.features.sun_shafts.add_passes(
      (&mut *graph, &mut *bindings, runtime),
      passes,
      view,
      (handles, lit),
      (light_volumes, is_froxeled),
      froxels,
    );
    self.features.rain.add_passes(
      (&mut *graph, &mut *bindings, runtime),
      passes,
      view,
      (
        handles,
        &lit,
        (
          shadowed.covers.map(|covers| covers.depth),
          shadowed.surface[2],
          shadowed.surface[0],
        ),
      ),
    );
    self
      .features
      .thunder
      .add_passes((&mut *graph, &mut *bindings, runtime), passes, view, handles);

    ResolveFeature::add_transparency_guide(&mut *graph, handles, (&mut ray_guides, before_water));

    self.features.flares.add_passes(
      (&mut *graph, &mut *bindings, runtime),
      passes.flares,
      (handles, lit),
      view,
    );
    self
      .features
      .fog
      .add_passes((&mut *graph, &mut *bindings, runtime), passes, view, (handles, lit));
    self
      .features
      .meter
      .add_passes((&mut *graph, &mut *bindings), passes, view, (handles, lit));

    let size: (u32, u32) = handles.size;
    let enhanced_bloom: Option<GraphTexture> = self.features.bloom.add_enhanced_passes(
      (&mut *graph, &mut *bindings, runtime),
      passes,
      view,
      (handles, size, lit.lighting),
    );

    // A selected composited surface, glass, marked where the present outlines the selection, once the enhanced
    // bloom has read the marks.
    if let (Some((draw, _)), Some(_)) = (&composited_draw, self.features.overlays.get_selection_color()) {
      self.features.composited.add_selection_passes(&mut *graph, passes, draw);
    }

    self
      .features
      .bloom
      .add_engine_passes((&mut *graph, runtime), passes, handles);

    graph.begin_group("resolve");

    let smoothed: GraphTexture =
      self
        .features
        .smoothing
        .add_passes((&mut *graph, &mut *bindings, runtime), passes, (handles.scene, size));
    let (mut resolved, mut shown): (GraphTexture, GraphTexture) = self.features.resolve.add_passes(
      (&mut *graph, &mut *bindings, runtime),
      passes,
      view,
      (
        // The water's pixels carried by its surface rather than by what lies under it.
        ViewTargetHandles {
          depth: depth_with_water.unwrap_or(handles.depth),
          ..handles
        },
        smoothed,
        opaque,
        ray_guides,
      ),
    );

    // The enhanced streaks over what the resolve shows, so no history carries them; the frame as drawn is that image
    // too where nothing rescaled it.
    if let Some(layer) = self.features.rain.add_over_resolved(
      (&mut *graph, &mut *bindings, runtime),
      passes,
      view,
      (
        handles,
        &lit,
        (
          shadowed.covers.map(|covers| covers.depth),
          shadowed.surface[2],
          shadowed.surface[0],
        ),
      ),
      (shown, self.features.resolve.get_shown_size(handles.size)),
    ) {
      if !self.features.resolve.is_rescaled() {
        resolved = layer;
      }

      shown = layer;
    }

    self.features.exposure.add_passes(
      (&mut *graph, &mut *bindings, runtime),
      passes,
      (handles.high, handles.scene, handles.size, lit.exposure),
    );

    output.scene = resolved;
    output.shown = shown;
    output.indirect_light = lit_frame.indirect_light.unwrap_or(record.unlit);
    output.reflections = lit_frame.reflections.unwrap_or(record.untraced);
    output.reflection_ends = lit_frame.reflection_ends.unwrap_or(record.no_ends);
    output.bloom = enhanced_bloom.unwrap_or(handles.bloom[0]);
  }

  /// Asks for a pick's texel back, its frame just submitted.
  pub fn request_pick(&self, slot: usize) {
    self.features.pick.request(slot);
  }

  /// A pick's texel once it is back; a view made since the pick has no readback for it, and it met nothing still drawn.
  pub fn take_pick(&self, slot: usize) -> Option<XrfResult<[u32; 4]>> {
    self.features.pick.take(slot)
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

  /// Where the next frame's samples sit within their pixels, in drawn pixels, `y` down, on a GPU that has DLSS where
  /// `has_dlss`: jittered while it resolves frames temporally and shows the finished frame, still otherwise.
  pub fn next_jitter(&mut self, options: &RenderViewOptions, ratio: f32, has_dlss: bool) -> Vec2 {
    self.features.resolve.next_jitter(options, ratio, has_dlss)
  }

  /// The mip bias the surfaces' textures are read at this frame, as `next_jitter` decided it.
  pub fn get_texture_bias(&self) -> f32 {
    self.features.resolve.get_texture_bias()
  }

  /// Asks for the counts recorded with the frame just submitted.
  pub fn request_stats(&self) {
    self.features.statics.request_stats();
    self.features.grass.request_stats();
    self.features.lights.request_report();
    self.features.reflections.request_demand();
    self.features.meter.request();
    self.features.exposure.request_readback();
    self.features.sky.request_readback();
  }

  /// Drops what a frame that was never submitted copied out for reading back, its counts and its pick, and forgets the
  /// periodic work it drew, which draws again.
  pub fn abandon_frame(&mut self) {
    self.features.statics.abandon_stats();
    self.features.statics.abandon_frame();
    self.features.resolve.abandon_frame();
    self.features.grass.abandon_stats();
    self.features.lights.abandon_report();
    self.features.reflections.abandon_demand();
    self.features.meter.abandon();
    self.features.exposure.abandon_readback();
    self.features.pick.abandon();
    // What the frame drew of the periodic work was never drawn.
    self.features.shadows.abandon_frame();
    self.features.surroundings.abandon_frame();
    self.features.sky.abandon_frame();

    if let Some(rain_cover) = &mut self.features.rain_cover {
      rain_cover.abandon_frame();
    }

    if let Some(level_surface) = &mut self.features.level_surface {
      level_surface.abandon_frame();
    }
  }

  /// What its frames are drawn with, as resolved from what `options` asked; the weather's light is the viewport's to
  /// add.
  pub fn describe_applied(&self, options: &RenderViewOptions) -> RenderAppliedReport {
    let antialiasing: RenderAntialiasing = self.get_antialiasing();
    let cascades: usize = self.shared.shadow_settings.get_cascade_count();

    RenderAppliedReport {
      antialiasing,
      is_ray_reconstructed: self.is_ray_reconstructed(),
      render_scale: self.features.resolve.get_upscaling().scale,
      shadows: (cascades > 0).then(|| RenderAppliedShadows {
        cascades: self.shared.shadow_settings.cascades[..cascades].to_vec(),
        resolution: self.features.shadows.get_maps().map_or(0, |maps| maps.resolution),
        filter: self.shared.shadow_settings.filter,
      }),
      ambient_occlusion: self
        .features
        .ambient_occlusion
        .is_drawn()
        .then_some(*self.features.ambient_occlusion.get_settings()),
      ambient_occlusion_method: self.features.ambient_occlusion.is_drawn().then(|| {
        match self.features.bitmask.get_search() {
          Some(search) if search.is_occluding && search.is_traced => RenderAmbientOcclusionMethod::Rtao,
          Some(search) if search.is_occluding => RenderAmbientOcclusionMethod::Vbao,
          _ => RenderAmbientOcclusionMethod::Gtao,
        }
      }),
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
          trace_size: if trace.is_raw {
            RenderTraceSize::Full
          } else {
            trace.trace_size
          },
          steps: self.features.reflections.get_settings().get_steps(),
        }),
      height_fog: self.features.fog.get_applied(),
      debanding: self.features.present.get_debanding(),
      bloom: if self.features.bloom.is_enhanced_drawn() {
        Some(RenderBloomMode::Enhanced)
      } else {
        self.features.bloom.is_engine_drawn().then_some(RenderBloomMode::Engine)
      },
      lights: Some(self.features.lights.get_settings()).filter(|settings| settings.is_enabled),
      grass: self.features.grass.get_applied(&options.features.grass),
      is_water: options.features.water.is_enabled,
      environment: None,
      sun: self
        .features
        .sky
        .get_sun_sprite()
        .and_then(|_| self.features.flares.get_shown())
        .map(str::to_owned),
    }
  }

  /// The static draws' pools and what the latest counted frame's cull kept and hid, and its lights.
  pub fn take_stats(&mut self, scene: &LevelScene) -> (RenderStaticReport, RenderLightsReport) {
    let [kept_clusters, kept_triangles, occluded_clusters, occluded_triangles] = self.features.statics.take_stats();
    let pools: RenderStaticReport = scene.statics.get_pools();
    let phases: u32 = if self.features.statics.get_cull().is_occluding != 0 {
      2
    } else {
      1
    };

    (
      RenderStaticReport {
        surface_list: RenderPoolUse {
          used: kept_clusters,
          ..pools.surface_list
        },
        commands: StaticBatch::list_deferred().count() as u32 * phases
          + u32::from(self.features.statics.get_cull().is_impostors != 0),
        kept_clusters,
        kept_triangles,
        occluded_clusters,
        occluded_triangles,
        ..pools
      },
      self.features.lights.take_report(),
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
    self
      .features
      .overlays
      .set((scene, device), (overlays, version), selection);
  }

  /// What a selection marks in the scene now, resolved again only once the target or the scene changed; none where
  /// nothing is selected or what it names is not in the scene yet.
  pub fn resolve_selection(
    &mut self,
    scene: &LevelScene,
    selection: Option<&RenderSelection>,
  ) -> Option<&StaticSelection> {
    self.features.overlays.resolve_selection(scene, selection)
  }

  /// What it draws over its frame, once made.
  pub fn get_overlays(&self) -> Option<&LevelOverlays> {
    self.features.overlays.get_overlays()
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
}
