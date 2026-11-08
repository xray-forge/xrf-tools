use std::time::Instant;

use glam::{Mat4, Vec4};

use crate::camera::camera_frustum::CameraFrustum;
use crate::contract::feature::lighting::model::render_light_level::RenderLightLevel;
use crate::contract::feature::weather::water::render_water_mode::RenderWaterMode;
use crate::contract::feature::weather::water::render_water_settings::RenderWaterSettings;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::contract::viewport::render_rect::RenderRect;
use crate::feature::effects::flares::sun_sprite::SunSprite;
use crate::feature::effects::rain::cover::rain_cover::RainCover;
use crate::feature::effects::rain::rain_uniform::RainUniform;
use crate::feature::effects::rain::rings::drop_rings::DropRings;
use crate::feature::image::present::present_inputs::PresentInputs;
use crate::feature::scene::drawable::draw::drawable_kind_source::DrawableKindSource;
use crate::feature::scene::drawable::drawable_trace_instance::DrawableTraceInstance;
use crate::feature::scene::grass::grass_view_frame::GrassViewFrame;
use crate::feature::tracing::reflections::trace::reflection_trace::ReflectionTrace;
use crate::feature::tracing::sun_trace::sun_trace_feature::SunTraceFeature;
use crate::feature::view::overhead_windows::OverheadWindows;
use crate::feature::view::scene_features::SceneFeatures;
use crate::feature::weather::fog::fog_feature::FogFeature;
use crate::feature::weather::water::water_frame::WaterFrame;
use crate::feature::weather::wet::puddles::level_surface::LevelSurface;
use crate::feature::weather::wet::puddles::surface_window::SurfaceWindow;
use crate::feature::weather::wet::wet_clock::WetClock;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::targets::screen_tiles::ScreenTiles;
use crate::pass::frame_lighting::lighting_frame::LightingFrame;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;
use crate::scene::level::level_prepare::LevelPrepare;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::lights::asks::light_asks::LightAsks;
use crate::scene::level::lights::asks::lights_frame::LightsFrame;
use crate::scene::level::view::scene_view::SceneView;
use crate::scene::level::view::view_prepare::ViewPrepare;
use crate::scene::level::view::view_time::ViewTime;
use crate::scene::static_scene::draw::static_sight::StaticSight;
use crate::scene::texture::texture_cache::TextureCache;

impl SceneView {
  /// The units the weather drives: the shadows' maps, the effects laid over the lit frame (whose flares give the sky its
  /// sun), the sky, the water, the wind, the falling rain and the wet surfaces following it, and the bloom. Answers what
  /// the lighting draws the sky with.
  pub(super) fn prepare_weather(
    &mut self,
    scene: &LevelScene,
    frame: &ViewPrepare<'_>,
    level: &LevelPrepare,
    now: Instant,
  ) -> LightingFrame {
    let (device, options, lighting) = (frame.device, frame.options, frame.lighting);
    let time: ViewTime = level.time;
    let size: Option<(u32, u32)> = self.targets.as_ref().map(|targets| (targets.width, targets.height));

    self.features.shadows.prepare(
      device,
      (options.features.shadows.get_cascade_count() > 0).then_some(options.features.shadows.resolution),
    );

    let sun_sprite: Option<SunSprite> = self.stages.effects.prepare(frame, level, now);

    self.features.sky.prepare(frame, sun_sprite, &scene.environments.1);
    self.features.water.prepare(
      device,
      options,
      WaterFrame {
        targets: self.targets.as_ref(),
        tiles: Self::get_water_tiles(scene, frame, level),
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
    let falling: Option<RainUniform> = self.features.rain.prepare(frame, level.gust, time, windows);

    self.features.rain.follow_display(self.shared.display);
    // The wet surfaces' splashes, flow and ripples run on the rain's clock while it falls; their drift on the view's.
    let wet_clock: WetClock = WetClock {
      splashes: match falling {
        Some(_) => self.features.rain.get_clock(),
        None => time.now.get_seconds(),
      },
      view: time,
    };

    self.features.wet.prepare(
      frame,
      &level.wetness,
      falling.as_ref(),
      wet_clock,
      windows,
      self.shared.resolve.noise_frame,
    );
    self.follow_rain_surfaces(windows);

    // The enhanced drops' rings, where they land on the level water's map, which the water and the puddles ripple by.
    let rings: DropRings = DropRings {
      tile: self.features.rain.get_rings(),
      reach: options.features.rain.ring_distance.max(0.0),
      strength: options.features.rain.ring_strength.max(0.0),
    };

    self.features.water.follow_rings(rings);
    self.features.wet.follow_rings(rings);

    let lighting_frame: LightingFrame = self.features.sky.get_lighting_frame(frame, &self.shared, time.now);

    self
      .features
      .bloom
      .prepare(frame.passes, options, lighting.engine, size);

    lighting_frame
  }

  /// The scene's units: the static cull; the grass, which reads its discard, plants around last frame's puddles and
  /// casts by the trace or the cascades; then what the views off the viewport draw the scene with. Answers the lights it
  /// sees by the cull's levels of detail, which the scene settles with every other view's.
  pub(super) fn prepare_scene(
    &mut self,
    scene: &LevelScene,
    frame: &ViewPrepare<'_>,
    lighting_frame: &LightingFrame,
    level: &LevelPrepare,
  ) -> LightAsks {
    let (device, options, lighting, view) = (frame.device, frame.options, frame.lighting, frame.view);
    let time: ViewTime = level.time;

    self
      .features
      .statics
      .prepare(device, &scene.statics, options, view, frame.drawn);

    let is_scene_traced: bool = frame.passes.tracing.scene_trace.is_traced() && scene.trace.is_some();
    let is_traced: bool = options.is_light_traced() && is_scene_traced;
    // The sun's cascades shadow the frame where they are asked for and the sun is not traced in their place.
    let is_sun_traced: bool = is_traced && SunTraceFeature::is_asked(options, self.shared.resolve);
    let is_cascaded: bool = options.features.shadows.get_cascade_count() > 0 && !is_sun_traced;

    let grass: GrassViewFrame<'_> = GrassViewFrame {
      camera: view,
      discard_below: self.features.statics.get_cull().discard_below,
      // The ambient effects blow the wind the grass reads this frame.
      time,
      is_windy: lighting.trees.is_some(),
      strength: level.gust.strength,
      foliage: self.features.wind.get_values(),
      // Where the puddles stood last frame, which no tuft is planted in: the wet readies after the grass.
      puddles: LevelSurface::get_grass_puddles(
        self.features.level_surface.as_ref(),
        frame.passes.weather.wet.get_dry()[1],
        &self.features.wet,
      ),
      is_traced,
      // The water readies before the scene, in the weather's units.
      is_reflected: is_scene_traced && self.features.water.is_fully_traced(),
      cascade_reach: is_cascaded
        .then(|| {
          options
            .features
            .shadows
            .get_reach(options.features.grass.shadow_cascades)
        })
        .flatten(),
    };

    self
      .features
      .grass
      .prepare(device, scene.grass.get_level(), &options.features.grass, &grass);
    self
      .features
      .water
      .follow_grass_cards(self.features.grass.get_trace_uniform().card_reach);

    let sway_amplitude: f32 = self.features.wind.get_sway_amplitude();
    let frame_lights: LightsFrame = LightsFrame {
      camera: *view,
      settings: options.features.lights,
      engine: lighting.engine,
      lod: (
        self.features.statics.get_cull().glod_start,
        self.features.statics.get_cull().glod_end,
      ),
      sway_amplitude,
      casters: self.features.grass.get_light_casters(),
      is_physical: options.is_light_physical(),
      is_specular_linear: options.is_specular_linear(),
      anchor: (options.get_physical_light_level() == Some(RenderLightLevel::Anchored))
        .then(|| lighting.get_ambient(&options.features.light_scales)),
      is_falloff_inverse_square: options.is_falloff_inverse_square(),
      light_scale: lighting_frame.light_units.local
        * lighting_frame.light_units.light_gain
        * lighting_frame.exposure.pre_exposure.current,
      time: time.now,
    };
    let lights: LightAsks = scene.lights.pick(frame_lights, &scene.campfire_shares);

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

    lights
  }

  /// The view's own units: the reflections, the view's trace, the composited surfaces in the `level`'s sight, the
  /// scene's stage and the image's.
  pub(super) fn prepare_view(
    &mut self,
    scene: &LevelScene,
    encoder: &mut FrameEncoder,
    frame: &ViewPrepare<'_>,
    level: &LevelPrepare,
  ) {
    self
      .features
      .reflections
      .prepare(frame, self.shared.resolve, &self.features.wet);
    let trace_instances: Vec<DrawableTraceInstance> =
      DrawableKindSource::list(&self.features.grass, scene, frame.passes)
        .iter()
        .filter_map(DrawableKindSource::get_trace_instance)
        .collect();

    self
      .features
      .view_trace
      .prepare(frame, &mut *encoder, scene.trace.as_ref(), &trace_instances);
    self
      .features
      .composited
      .prepare(frame, scene, &mut *encoder, &level.sight);
    self.stages.scene.prepare(frame.options, frame.view);
    self.stages.image.prepare(frame);
  }

  /// The lighting the passes read, the enhanced fog's height fog faded out indoors, and the lighting's stage, which
  /// reads it.
  pub(super) fn prepare_lighting(
    &mut self,
    frame: &ViewPrepare<'_>,
    lighting_frame: &LightingFrame,
    level: &LevelPrepare,
  ) {
    let (options, view) = (frame.options, frame.view);

    self.shared.lighting = LightingUniform::new(frame.lighting, view.view, options, lighting_frame);
    self.shared.lighting.height_fog.w *= FogFeature::get_indoor_share(level.camera_hemi);
    self.stages.lighting.prepare(frame, &self.shared);
  }

  /// Writes where the rain cover and the level's surface now stand into what reads them, once `commit_periodic` has
  /// moved them: written before, a frame redrawing one would read it through where it stood the frame before.
  pub(super) fn follow_overhead_maps(&mut self) {
    let windows: OverheadWindows = self.get_overhead_windows();

    self.features.rain.follow_cover(windows);
    self.features.wet.follow(windows);
    self.follow_rain_surfaces(windows);
  }

  /// Follows the rain to the level water's and surface's maps, which the enhanced drops land on while they are drawn.
  fn follow_rain_surfaces(&mut self, windows: OverheadWindows) {
    let is_drawn: bool = self.features.level_surface.is_some() && self.features.wet.is_enhanced();

    self.features.rain.follow_surfaces(is_drawn.then_some(windows));
  }

  /// Where the rain cover and the level's surface stand now.
  fn get_overhead_windows(&self) -> OverheadWindows {
    let cover: Option<&RainCover> = self.features.rain_cover.as_ref();
    let surface: Option<&LevelSurface> = self.features.level_surface.as_ref();
    let SurfaceWindow {
      window: surface_window,
      shape: surface_shape,
    } = surface.map_or(LevelSurface::get_unmade_window(), LevelSurface::get_window);

    OverheadWindows {
      cover: cover.map_or(RainCover::get_unmade_window(), RainCover::get_window),
      cover_shape: cover.map_or(RainCover::get_unmade_shape(), RainCover::get_shape),
      surface: surface_window,
      surface_shape,
      water: surface.map_or(LevelSurface::get_unmade_water_window(), LevelSurface::get_water_window),
    }
  }

  /// The tiles of the frame the level's water in sight may touch, by the spheres about its slots; none where the view
  /// sees none.
  fn get_water_tiles(scene: &LevelScene, frame: &ViewPrepare<'_>, level: &LevelPrepare) -> ScreenTiles {
    let mut tiles: ScreenTiles = ScreenTiles::default();

    if level.sight.is_water() {
      let (frustum, view_projection): (CameraFrustum, Mat4) =
        (frame.view.get_frustum(), frame.view.get_view_projection());

      for sphere in scene.statics.list_seen_spheres(&frustum, StaticSight::is_water) {
        tiles.cover_sphere(view_projection, sphere);
      }
    }

    tiles
  }

  /// Where on the frame the water, the particles, the enhanced rain's splashes and the distorting surfaces of the scene
  /// move what is seen through it, once every unit knows what it draws: every tile where the water or the splashes do,
  /// which keep no bound; none where nothing does.
  pub(super) fn get_distortion_tiles(&self, options: &RenderViewOptions) -> ScreenTiles {
    let features: &SceneFeatures = &self.features;

    if options.mode.is_wireframe {
      return ScreenTiles::default();
    }

    if self.is_water_distorting(options) || features.rain.is_distorting() {
      return ScreenTiles::new_whole();
    }

    let mut tiles: ScreenTiles = *features.particles.get_distortion_tiles();

    tiles.cover_tiles(features.composited.get_distortion_tiles());

    tiles
  }

  /// Whether this frame's distortion may blur the frame towards the engine's bloom, which is built for it then: where a
  /// writer drawn this frame writes some blue, the water, or a particle or a surface of the `scene` whose map holds some
  /// or is not yet up in `textures`; the enhanced rain's splashes write none.
  pub(super) fn is_distortion_blurring(
    &self,
    scene: &LevelScene,
    options: &RenderViewOptions,
    textures: &TextureCache,
  ) -> bool {
    let features: &SceneFeatures = &self.features;

    if options.mode.is_wireframe {
      return false;
    }

    self.is_water_distorting(options)
      || features.particles.is_blurring(textures)
      || (!features.composited.get_distortion_tiles().is_empty()
        && scene.statics.surfaces.is_distortion_blurring(textures))
  }

  /// Whether water draws its distortion this frame; the enhanced water refracts what is behind it itself and writes
  /// none.
  fn is_water_distorting(&self, options: &RenderViewOptions) -> bool {
    let water: &RenderWaterSettings = &options.features.water;

    self.features.water.is_drawn()
      && water.is_distorted
      && water.mode != RenderWaterMode::Enhanced
      && options.mode.is_lit
  }

  /// Notes what the window pass reads, once every other unit knows what it draws: where the frame distorts
  /// (`distortion`), which bloom it lays over the frame, and the film's grain, drawn for `noise_frame`.
  pub(super) fn prepare_present(
    &mut self,
    options: &RenderViewOptions,
    output: RenderRect,
    noise_frame: u32,
    distortion: ScreenTiles,
  ) {
    if self.targets.is_none() {
      return;
    }

    let features: &SceneFeatures = &self.features;
    let inputs: PresentInputs = PresentInputs {
      is_upscaled: features.resolve.is_rescaled(),
      is_occluded: self.stages.lighting.is_occluded(),
      is_indirect: self.stages.lighting.is_indirect(),
      reflection_ratio: features.reflections.get_trace().map(ReflectionTrace::get_ratio),
      distortion,
      selection_color: features.overlays.get_selection_color(),
      marks: features.helpers.get_present_marks(),
      is_engine_bloomed: features.bloom.is_engine_drawn(),
      is_enhanced_bloomed: features.bloom.is_enhanced_drawn(),
      is_engine_built: features.bloom.is_engine_built(),
      grass_reach: features.grass.get_cascade_reach(),
      light: self.shared.lighting.get_light_encoding(),
      display: self.shared.display,
      grain_frame: noise_frame,
    };

    self.features.present.prepare(options, output, inputs);
  }
}
