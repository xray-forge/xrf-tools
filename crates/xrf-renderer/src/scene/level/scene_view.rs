use std::sync::Arc;
use std::time::Instant;

use glam::{Mat4, Vec2, Vec3, Vec4};
use xrf_error::XrfResult;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphColorAttachment, GraphRuntime, GraphTexture, GraphTextureDescriptor,
  StorageArray, StorageArrayMut, StorageValue, UniformBinding,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_applied_indirect_light::RenderAppliedIndirectLight;
use crate::contract::render_applied_reflections::RenderAppliedReflections;
use crate::contract::render_applied_report::RenderAppliedReport;
use crate::contract::render_applied_shadows::RenderAppliedShadows;
use crate::contract::render_bloom_mode::RenderBloomMode;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_level_hit::RenderLevelHit;
use crate::contract::render_lights_report::RenderLightsReport;
use crate::contract::render_overlay::RenderOverlay;
use crate::contract::render_particles_report::RenderParticlesReport;
use crate::contract::render_pool_use::RenderPoolUse;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_selection::RenderSelection;
use crate::contract::render_static_report::RenderStaticReport;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_water_settings::RenderWaterSettings;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic_scheduler::PeriodicScheduler;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::texture_copy::add_texture_copy;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_target_uses::ViewTargetUses;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::lighting::ambient_gust::AmbientGust;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::composited_parameters::CompositedParameters;
use crate::pass::contact_shadow_uniform::ContactShadowUniform;
use crate::pass::froxel_uniform::FroxelUniform;
use crate::pass::grass_colour_parameters::GrassColourParameters;
use crate::pass::grass_puddles_uniform::GrassPuddlesUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::rain_uniform::RainUniform;
use crate::pass::reflection_trace::ReflectionTrace;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::sun_parameters::SunParameters;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::view_binding::ViewBinding;
use crate::pass::water_draw::WaterDraw;
use crate::pass::water_uniform::WaterUniform;
use crate::pass::wet_uniform::WetUniform;
use crate::scene::level::cover_textures::CoverTextures;
use crate::scene::level::feature::composited_draw::CompositedDraw;
use crate::scene::level::feature::light_buffers::LightBuffers;
use crate::scene::level::feature::lights_feature::LightsFeature;
use crate::scene::level::feature::overhead_windows::OverheadWindows;
use crate::scene::level::feature::present_inputs::PresentInputs;
use crate::scene::level::feature::scene_features::SceneFeatures;
use crate::scene::level::feature::water_frame::WaterFrame;
use crate::scene::level::grass_level::GrassLevel;
use crate::scene::level::level_frame::LevelFrame;
use crate::scene::level::level_overlays::LevelOverlays;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::level_surface::{LevelSurface, PuddleKeepInputs};
use crate::scene::level::light_shadow_atlas::LightShadowAtlas;
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::level::lights_frame::LightsFrame;
use crate::scene::level::rain_cover::RainCover;
use crate::scene::level::scene_output::SceneOutput;
use crate::scene::level::shadow_frame::ShadowFrame;
use crate::scene::level::shadow_sway::ShadowSway;
use crate::scene::level::surroundings_cube::SurroundingsCube;
use crate::scene::level::view_frame::ViewFrame;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::static_scene::static_selection::StaticSelection;
use crate::scene::static_scene::static_sight::StaticSight;
use crate::scene::static_scene::static_slot_info::StaticSlotInfo;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// The camera hemi under which the enhanced fog's height fog is gone and over which it is whole, about the engine's
/// indoors at `0.05` (`CGamePersistent::WeathersUpdate`).
const INDOOR_HEMI_FADE: (f32, f32) = (0.03, 0.1);

/// What a pick's texel says it met: a cluster, by its index and place, or an impostor, by its index.
const PICKED_CLUSTER: u32 = 1;
const PICKED_IMPOSTOR: u32 = 2;

/// A scene as one viewport draws it: the values its units share this frame (`ViewFrame`), its targets, and its units
/// (`SceneFeatures`), which each keep what they keep between frames. The scene (`LevelScene`) is shared by every view of
/// it and passed in; this prepares the view's frame from it and records it into the frame's graph.
pub struct SceneView {
  /// The values every unit reads this frame.
  shared: ViewFrame,
  /// The G-buffer, depth, light and scene it draws into, once its size is known.
  targets: Option<ViewTargets>,
  /// Its feature units.
  features: SceneFeatures,
}

impl SceneView {
  /// What the scene's particles came to since the last report, as this view draws them.
  pub fn take_particles_report(&self, scene: &mut LevelScene) -> RenderParticlesReport {
    scene.particles.take_report(self.features.particles.drawn)
  }

  /// What it keeps from one frame to the next.
  /// The size its scene is rendered at, once its targets are made.
  pub fn get_render_size(&self) -> Option<(u32, u32)> {
    self.targets.as_ref().map(|it| (it.width, it.height))
  }

  /// This frame's unjittered view projection and the last one's, which the surfaces' motion is measured between.
  pub fn next_motion(&mut self, current: Mat4) -> (Mat4, Mat4) {
    self.features.resolve.next_motion(current)
  }

  /// The last frame's jittered view projection, this frame's taking its place.
  pub fn next_jittered(&mut self, current: Mat4) -> Mat4 {
    self.features.resolve.next_jittered(current)
  }

  /// A view of `scene`, nothing of it drawn yet.
  pub fn new(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    view_layout: &wgpu::BindGroupLayout,
    scene: &LevelScene,
  ) -> Self {
    Self {
      shared: ViewFrame::default(),
      targets: None,
      features: SceneFeatures::new((device, queue), view_layout, scene.statics.cull.args.size()),
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
  ) {
    profiling::scope!("prepare view");

    if !self.targets.as_ref().is_some_and(|it| it.is_sized(width, height)) {
      let targets: ViewTargets = ViewTargets::new(device, width, height);
      self.targets = Some(targets);
      self.features.statics.fit(device, (width, height));
    }

    self.features.overlays.follow_skeletons((scene, device));

    self.features.shadows.prepare(
      device,
      options.features.shadows.resolution,
      options.features.shadows.get_cascade_count() > 0,
    );

    self
      .features
      .exposure
      .prepare(queue, &options.features.exposure, Instant::now());

    let sun_sprite: Option<(String, Vec4)> = self.features.flares.prepare(
      device,
      (lighting, weather),
      options,
      weather_textures,
      (view, weather_rate),
      self.targets.is_some(),
    );

    let sky = &lighting.sky;

    self.features.sky.prepare(
      device,
      (passes, options),
      (sky, sun_sprite, &scene.environments.1),
      weather_textures,
    );

    // The ambient effects blow the wind the grass, the rain and the campfires read this frame.
    let gust: AmbientGust = scene.gust;

    // What of the scene drawn by passes of its own the view may see: none of the water's or the surface passes' work
    // runs for what is out of sight.
    let sight: StaticSight = scene.statics.get_sight(&view.get_planes());

    self.features.water.prepare(
      device,
      options,
      WaterFrame {
        targets: self.targets.as_ref(),
        is_in_view: sight.is_water(),
        intensity: lighting.water_intensity,
        wind: lighting.wind,
        rain: lighting.rain.map_or(0.0, |rain| rain.density),
        time: scene.started.elapsed().as_secs_f32(),
      },
    );

    self
      .features
      .wind
      .prepare(options, lighting, scene.started.elapsed().as_secs_f32());
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
      (windows, self.features.resolve.get_noise_frame()),
      weather_textures,
    );
    self
      .features
      .thunder
      .prepare(device, (lighting, weather), options, weather_textures);

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
        .features
        .sky
        .get_sun_sprite()
        .filter(|(texture, _)| weather_textures.is_up(texture))
        .map_or(Vec4::ZERO, |(_, sprite)| *sprite),
    };

    self.features.sun_shafts.prepare(options, lighting);
    self.features.bloom.prepare(
      passes,
      options,
      lighting.engine,
      self.targets.as_ref().map(|targets| (targets.width, targets.height)),
    );
    self.features.fog.prepare(
      options,
      lighting,
      self.targets.as_ref().map(|targets| (targets.width, targets.height)),
      scene.started.elapsed().as_secs_f32(),
    );

    self
      .features
      .statics
      .prepare(&scene.statics, options, (view, field_of_view), (width, height));
    self.features.grass.prepare(
      (device, &mut *encoder),
      (passes.grass, scene.grass.get_level()),
      &options.features.grass,
      view,
      self.features.statics.get_cull().discard_below,
      (
        scene.started.elapsed().as_secs_f32(),
        lighting.trees.is_some(),
        gust.strength,
      ),
      self.features.wind.get_values(),
      // Where the puddles stood last frame, which no tuft is planted in: the wet readies after the grass.
      (
        self
          .features
          .level_surface
          .as_ref()
          .map_or(passes.wet.get_dry()[1], |surface| &surface.puddles),
        GrassPuddlesUniform::new(
          self
            .features
            .level_surface
            .as_ref()
            .map_or(LevelSurface::get_unmade().0, LevelSurface::get_window),
          self.features.wet.has_puddles(),
        ),
      ),
    );
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
        sway: &to_sway(&scene.statics, self.features.wind.get_sway()),
        campfire_shares: &scene.campfire_shares,
      },
    );
    self.shared.camera = *view;
    self.shared.sun_direction = lighting.get_sun_direction();
    self.shared.switches = Vec4::new(
      1.0,
      options.mode.is_bumped as u32 as f32,
      options.features.hemi_strength,
      0.0,
    );
    self.shared.shadow_settings = options.features.shadows.clone();
    self.shared.is_wireframe = options.mode.is_wireframe;
    self.features.resolve.prepare(
      device,
      view,
      (
        self.targets.as_ref().map(|targets| (targets.width, targets.height)),
        output,
      ),
      (options.output.upscaling, options.is_scene_linear()),
    );
    self.features.smoothing.prepare(
      passes,
      options.features.antialiasing,
      (
        self.targets.is_some() && options.mode.debug_view == RenderDebugView::Final,
        options.is_scene_linear(),
      ),
    );
    self.features.lights.settings = options.features.lights;

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

    self.shared.lighting = LightingUniform::new(lighting, view.view, options, &frame);
    // The enhanced fog's height fog fades out as the camera goes indoors, by its hemi as the engine tells indoors.
    self.shared.lighting.height_fog.w *= scene.camera_hemi.map_or(1.0, |hemi| {
      let share: f32 = ((hemi - INDOOR_HEMI_FADE.0) / (INDOOR_HEMI_FADE.1 - INDOOR_HEMI_FADE.0)).clamp(0.0, 1.0);

      share * share * (3.0 - 2.0 * share)
    });

    self.features.contact_shadows.prepare(
      options,
      self.shared.lighting.to_sun,
      (height, self.features.resolve.get_noise_frame()),
    );

    scene.particles.fill(view, options, &mut self.features.particles);

    self.features.particles.upload((device, &mut *encoder));

    self.prepare_present(options, output, scene.started.elapsed().as_secs_f32());
    self.features.volumetric.prepare(device, options);
  }

  /// Writes where the rain cover and the level's surface now stand into what reads them, once `commit_periodic` has
  /// moved them: written before, a frame redrawing one would read it through where it stood the frame before.
  fn follow_overhead_maps(&mut self) {
    let windows: OverheadWindows = self.get_overhead_windows();

    self.features.rain.follow_cover(windows.cover, windows.cover_shape);
    self.features.wet.follow(windows);
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
    (view_layout, textures): (&wgpu::BindGroupLayout, &TextureCache),
  ) {
    let SceneView { shared, features, .. } = self;
    let SceneFeatures {
      shadows,
      rain_cover,
      level_surface,
      overhead_layout: (overhead_view_layout, args_size),
      surroundings,
      reflections,
      rain,
      wet,
      sky,
      statics: statics_feature,
      ..
    } = features;
    let statics: &StaticScene = &scene.statics;
    let frame: ShadowFrame<'_> = ShadowFrame {
      scene: statics,
      camera: &shared.camera,
      settings: &shared.shadow_settings,
      sun_direction: shared.sun_direction,
      switches: shared.switches,
      sway: to_sway(statics, features.wind.get_sway()),
      params: statics_feature.get_cull(),
      textures,
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
        .get_or_insert_with(|| RainCover::new(device, overhead_view_layout, *args_size))
        .offer(&frame, wet.get_reach(), scheduler);
    } else {
      *rain_cover = None;
    }

    // The puddles seat on it.
    if wet.is_enhanced() {
      level_surface
        .get_or_insert_with(|| LevelSurface::new(device, overhead_view_layout, *args_size))
        .offer(&frame, scene.lights.get_fires(), scheduler);
    } else {
      *level_surface = None;
    }

    sky.offer_refill(scheduler);
  }

  /// Readies what `scheduler` granted of what `offer_periodic` offered, which the frame's passes then draw, and writes
  /// where the overhead maps now stand; `encoder` takes the copies a list growing makes.
  pub fn commit_periodic(
    &mut self,
    scene: &LevelScene,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    scheduler: &PeriodicScheduler,
    textures: &TextureCache,
  ) {
    let SceneView { shared, features, .. } = self;
    let SceneFeatures {
      shadows,
      rain_cover,
      level_surface,
      surroundings,
      reflections,
      sky,
      statics: statics_feature,
      ..
    } = features;
    let statics: &StaticScene = &scene.statics;
    let frame: ShadowFrame<'_> = ShadowFrame {
      scene: statics,
      camera: &shared.camera,
      settings: &shared.shadow_settings,
      sun_direction: shared.sun_direction,
      switches: shared.switches,
      sway: to_sway(statics, features.wind.get_sway()),
      params: statics_feature.get_cull(),
      textures,
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
    textures: &TextureCache,
    scheduler: &PeriodicScheduler,
  ) {
    let LevelScene { statics, lights, .. } = scene;
    let frame: ShadowFrame<'_> = ShadowFrame {
      scene: statics,
      camera: &self.shared.camera,
      settings: &self.shared.shadow_settings,
      sun_direction: self.shared.sun_direction,
      switches: self.shared.switches,
      sway: to_sway(statics, self.features.wind.get_sway()),
      params: self.features.statics.get_cull(),
      textures,
    };

    lights.commit_shadows(device, encoder, &frame, scheduler);
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
  ) -> Option<GraphTexture> {
    let lights: &'a LightsFeature = &self.features.lights;
    let buffers: LightBuffers = lights.add_binning_passes((&mut *graph, &mut *bindings, &mut *runtime), passes);

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

    lights.add_draw_passes(
      (&mut *graph, &mut *bindings),
      passes,
      (view, textures, scene.lights.get_shadow_atlas()),
      (targets, lit, buffers),
      (contact, self.features.contact_shadows.get_light_count()),
    );
    lights.add_overflow_pass(&mut *graph, buffers);

    // GTAO searches on its own; VBAO, the indirect light or both share one bitmask search.
    self
      .features
      .ambient_occlusion
      .add_passes((&mut *graph, &mut *bindings, &mut *runtime), passes, view, targets);

    self
      .features
      .bitmask
      .add_passes((graph, bindings, runtime), passes, view, targets)
  }

  /// Ends a frame its graph recorded: the water's reflection, the bitmask search's accumulations and the temporal
  /// resolve's history it wrote become the ones the next frame keeps.
  pub fn end_frame(&mut self, frame: &LevelFrame) {
    if frame.is_lit {
      self.features.water.finish_frame();

      self.features.bitmask.end_frame();

      if let Some(targets) = &mut self.targets {
        self.features.reflections.end_frame(&mut targets.depth);
      }

      self.features.volumetric.end_frame();
    }

    self.features.resolve.end_frame();
  }

  /// Notes what the window pass reads, once every other unit knows what it draws: whether the water, a particle or a
  /// distorting surface of the scene moves what is seen through it, which bloom it lays over the frame, and the sky
  /// debanding, its noise turning with `time`.
  fn prepare_present(&mut self, options: &RenderViewOptions, output: RenderRect, time: f32) {
    let Some(render) = self.targets.as_ref().map(|targets| (targets.width, targets.height)) else {
      return;
    };
    let water: &RenderWaterSettings = &options.features.water;
    let is_water_distorting: bool = water.is_enabled && water.is_distorted && options.mode.is_lit;
    let features: &SceneFeatures = &self.features;
    let inputs: PresentInputs = PresentInputs {
      is_occluded: features.ambient_occlusion.is_drawn(),
      is_indirect: features.bitmask.get_search().is_some_and(|search| search.is_lit),
      reflection_ratio: features.reflections.get_trace().map(ReflectionTrace::get_ratio),
      is_distorting: !options.mode.is_wireframe
        && (is_water_distorting || features.particles.is_distorting() || features.composited.is_distorting()),
      selection_color: features.overlays.get_selection_color(),
      is_engine_bloomed: features.bloom.is_engine_drawn(),
      is_enhanced_bloomed: features.bloom.is_enhanced_drawn(),
    };

    self.features.present.prepare(options, (render, output), inputs, time);
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
      is_lit: self.targets.is_some(),
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

    let pick_slot: Option<usize> = frame.pick_slot;
    let is_pick_through: bool = frame.is_pick_through;
    let is_drawn: bool = !self.shared.is_wireframe;
    let is_raining: bool = self.features.rain.is_raining();
    let is_wet: bool = self.features.wet.is_wet();
    let is_lit: bool = frame.is_lit;
    // The shadowed lights' shafts, drawn into a target of their own for the sun shafts' composite to add.
    let is_froxeled: bool = is_lit && self.features.volumetric.is_froxeled();
    let light_atlas: LightShadowAtlas<'a> = scene.lights.get_shadow_atlas();
    let has_sky: bool = self.features.sky.get_views().is_some();
    let is_composited: bool = is_drawn && has_sky;
    let has_particles: bool = is_drawn && self.features.particles.is_drawing();
    let particle_surfaces: &wgpu::Buffer = scene.particles.get_surfaces();
    let texture_group: &wgpu::BindGroup = textures.get_bind_group();
    let scene_view: &'a SceneView = self;
    let Some(targets) = &scene_view.targets else {
      return None;
    };
    let unlit: GraphTexture = bindings.import_view(&mut *graph, "indirect light none", passes.vbao.get_dark());
    let untraced: GraphTexture =
      bindings.import_view(&mut *graph, "reflections none", passes.reflections.get_untraced());
    let no_ends: GraphTexture =
      bindings.import_view(&mut *graph, "reflection ends none", passes.reflections.get_no_ends());
    let uses: ViewTargetUses = ViewTargetUses {
      is_occluded: scene_view.features.ambient_occlusion.is_drawn(),
      is_hazed: scene_view.features.sky.is_hazing(),
      is_engine_bloomed: scene_view.features.bloom.is_engine_drawn(),
    };
    let handles: ViewTargetHandles =
      ViewTargetHandles::import(&mut *graph, &mut *bindings, (targets, uses, passes.vbao.get_dark()));
    let statics: StaticSceneHandles = scene_view.features.statics.import(
      (&mut *graph, &mut *bindings, runtime),
      &scene.statics,
      scene_view.features.wind.get_uniform(),
    )?;
    let layouts: [StaticDrawParameters; StaticLayout::COUNT] = statics.get_camera_draws().layouts;
    let lit: LightingHandles = LightingHandles {
      lighting: runtime.push_uniform(&scene_view.shared.lighting),
      exposure: bindings.import_buffer(&mut *graph, "exposure", &scene_view.features.exposure.state),
      material_lut: bindings.import_view(&mut *graph, "material table", &passes.table.view),
      lut_sampler: &passes.table.sampler,
      shadow_maps: bindings.import_view(
        &mut *graph,
        "sun shadow maps",
        scene_view
          .features
          .shadows
          .get_maps()
          .map_or(passes.sun.get_unshadowed(), |maps| &maps.view),
      ),
      shadows: runtime.push_uniform(scene_view.features.shadows.get_values()),
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
        .features
        .grass
        .add_planting(&mut *graph, &mut *bindings, (passes.grass, grass_level));

    scene_view.features.statics.add_gbuffer_passes(
      (&mut *graph, &mut *bindings),
      passes,
      (view, texture_group, &statics),
      handles,
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

    scene_view.features.statics.add_stats_pass(&mut *graph, &statics);

    if let (true, Some(grass_args)) = (is_drawn, grass_args) {
      scene_view.features.grass.add_draw(
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

    if is_drawn {
      scene_view
        .features
        .statics
        .add_wallmark_passes(&mut *graph, passes, (view, texture_group, &statics), handles);
    }

    graph.begin_group("shadows");

    scene_view.features.shadows.add_passes(
      (&mut *graph, &mut *bindings),
      passes,
      (&statics, lit.shadow_maps),
      (scene_view.features.statics.get_cull(), texture_group),
    );

    // The rain's cover, which its passes draw and reduce and the rain and the wet surfaces read.
    let covers: Option<CoverTextures> = scene_view
      .features
      .rain_cover
      .as_ref()
      .filter(|_| is_raining || is_wet)
      .map(|rain_cover| {
        rain_cover.add_passes(
          (&mut *graph, &mut *bindings, &mut *runtime),
          passes,
          &statics,
          (scene_view.features.statics.get_cull(), texture_group),
        )
      });
    let cover: Option<GraphTexture> = covers.map(|covers| covers.depth);
    let wet: Option<UniformBinding<WetUniform>> = scene_view.features.wet.push_uniform(runtime);

    // The level's surface seen from overhead, its lowest heights, its puddle sites and its water, which the enhanced
    // wetting places its puddles by.
    let [surface_lowest, puddles, surface_water]: [GraphTexture; 3] = match (wet, &scene_view.features.level_surface) {
      (Some(wet), Some(level_surface)) if scene_view.features.wet.is_enhanced() => {
        let water: UniformBinding<WaterUniform> = runtime.push_uniform(scene_view.features.water.get_uniform());
        let keep: PuddleKeepInputs = PuddleKeepInputs {
          wet,
          noise: &passes.water.get_maps().perlin,
          sampler: passes.wet.get_sampler(),
        };

        level_surface.add_passes(
          (&mut *graph, &mut *bindings, runtime),
          passes,
          &statics,
          (scene_view.features.statics.get_cull(), texture_group),
          (water, keep),
        )
      }
      _ => {
        let [lowest, puddles, water]: [&'a wgpu::TextureView; 3] = passes.wet.get_dry();

        [
          bindings.import_view(&mut *graph, "level surface lowest", lowest),
          bindings.import_view(&mut *graph, "puddles", puddles),
          bindings.import_view(&mut *graph, "level water", water),
        ]
      }
    };

    if frame.is_scene_first {
      scene.lights.add_shadow_passes(
        (&mut *graph, &mut *bindings),
        passes,
        &statics,
        (scene_view.features.statics.get_cull(), texture_group),
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
      );

      let surroundings: &SurroundingsCube = &scene_view.features.surroundings;
      let sky: Option<SkyParameters> = scene_view.features.sky.add_passes(
        (&mut *graph, &mut *bindings, runtime),
        passes,
        (handles, lit),
        surroundings.is_active().then_some((
          surroundings,
          &statics,
          (scene_view.features.statics.get_cull(), texture_group),
        )),
      );
      // The sky's group, which the water's scene pass reads too once the water's own sky views shadow the name.
      let sky_parameters: Option<SkyParameters> = sky;

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
        &lit,
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

        scene_view.features.combine.add_passes(
          &mut *graph,
          passes,
          view,
          (handles, lit, sky),
          [
            indirect_light.unwrap_or(unlit),
            reflections.unwrap_or(untraced),
            wet_surface,
            wet_reach,
          ],
        );
      }

      let scene: GraphTexture = handles.scene;
      let water_sky = scene_view
        .features
        .sky
        .get_views()
        .filter(|_| scene_view.features.water.is_drawn());
      // The frame as it stands before the water, the composited surfaces and the particles draw, copied once for all
      // that read it: the enhanced water refracts and reflects it, and the temporal resolve's reactive weight is what
      // the rest change of it.
      let before_water: Option<GraphTexture> = (scene_view.features.resolve.is_reading_scene_before_water()
        || (water_sky.is_some() && scene_view.features.water.is_enhanced()))
      .then(|| {
        let (width, height) = handles.size;
        let copy: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
          "scene before water",
          width,
          height,
          ViewTargets::SCENE,
        ));

        add_texture_copy(&mut *graph, "scene before water", scene, copy);

        copy
      });
      let opaque: Option<GraphTexture> = scene_view
        .features
        .resolve
        .add_reactive_passes((&mut *graph, &mut *bindings), (scene, before_water));

      if let Some(sky) = water_sky {
        let draw: WaterDraw<'_> = WaterDraw {
          targets: handles,
          before_water: before_water.filter(|_| scene_view.features.water.is_enhanced()),
          view,
          textures: texture_group,
          layouts,
          args: scene_view.features.statics.list_draw_args(&statics),
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
            args: scene_view.features.statics.list_draw_args(&statics),
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
        scene_view.features.particles.add_passes(
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
        (handles, &scene_view.features.lights, light_atlas),
      );
      let froxels: Option<(UniformBinding<FroxelUniform>, GraphTexture)> =
        scene_view.features.volumetric.add_froxel_passes(
          (&mut *graph, &mut *bindings, runtime),
          passes,
          view,
          (lit, light_atlas, &scene_view.features.lights),
        );

      scene_view.features.sun_shafts.add_passes(
        (&mut *graph, &mut *bindings, runtime),
        passes,
        view,
        (handles, lit),
        (light_volumes, is_froxeled),
        froxels,
      );

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
      if let (Some((draw, _)), Some(_)) = (&composited_draw, scene_view.features.overlays.get_selection_color()) {
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

    if let Some(slot) = pick_slot {
      scene_view.features.pick.add_passes(
        (&mut *graph, &mut *bindings),
        passes.gbuffer,
        (texture_group, &statics),
        (slot, is_pick_through),
      );
    }

    Some(SceneOutput {
      targets: handles,
      scene: frame,
      shown: shown.unwrap_or(frame),
      present: runtime.push_uniform(scene_view.features.present.get_uniform()),
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
    lit: &LightingHandles<'a>,
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
              &self.features.lights.record_buffer,
            )),
            trace_light_view: runtime.push_uniform(&self.features.lights.uniform),
            trace_shadow_atlas: bindings.import_view(
              &mut *graph,
              "light shadow atlas",
              scene.lights.get_shadow_atlas().view,
            ),
            trace_shadow_sampler: scene.lights.get_shadow_atlas().sampler,
            trace_sun_maps: lit.shadow_maps,
            trace_sun_shadows: lit.shadows,
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
            trace_grass: runtime.push_uniform(self.features.grass.get_trace_uniform()),
          },
          texture_group,
        )
      })
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

  /// Where the next frame's samples sit within their pixels, in drawn pixels, `y` down: jittered while it resolves
  /// frames temporally and shows the finished frame, still otherwise.
  pub fn next_jitter(&mut self, options: &RenderViewOptions, ratio: f32) -> Vec2 {
    self.features.resolve.next_jitter(options, ratio)
  }

  /// Asks for the counts recorded with the frame just submitted.
  pub fn request_stats(&self) {
    self.features.statics.request_stats();
    self.features.lights.request_report();
  }

  /// Drops what a frame that was never submitted copied out for reading back, its counts and its pick, and forgets the
  /// periodic work it drew, which draws again.
  pub fn abandon_frame(&mut self) {
    self.features.statics.abandon_stats();
    self.features.lights.abandon_report();
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

  /// What its frames are drawn with, as resolved from what `options` asked; the weather's light is the viewport's to add.
  pub fn describe_applied(&self, options: &RenderViewOptions) -> RenderAppliedReport {
    let antialiasing: RenderAntialiasing = self
      .features
      .resolve
      .get_temporal()
      .or(self.features.smoothing.get_mode())
      .unwrap_or(RenderAntialiasing::None);
    let cascades: usize = self.shared.shadow_settings.get_cascade_count();

    RenderAppliedReport {
      antialiasing,
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
      debanding: self.features.present.get_debanding(),
      bloom: if self.features.bloom.is_enhanced_drawn() {
        Some(RenderBloomMode::Enhanced)
      } else {
        self.features.bloom.is_engine_drawn().then_some(RenderBloomMode::Engine)
      },
      lights: self
        .features
        .lights
        .settings
        .is_enabled
        .then_some(self.features.lights.settings),
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
