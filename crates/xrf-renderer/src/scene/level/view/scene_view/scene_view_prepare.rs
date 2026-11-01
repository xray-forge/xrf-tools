use std::time::Instant;

use glam::Vec4;

use crate::contract::feature::lighting::model::render_light_level::RenderLightLevel;
use crate::contract::feature::weather::water::render_water_mode::RenderWaterMode;
use crate::contract::feature::weather::water::render_water_settings::RenderWaterSettings;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::contract::viewport::render_rect::RenderRect;
use crate::feature::effects::flares::sun_sprite::SunSprite;
use crate::feature::effects::rain::cover::rain_cover::RainCover;
use crate::feature::effects::rain::rain_uniform::RainUniform;
use crate::feature::effects::rain::rings::drop_rings::DropRings;
use crate::feature::image::present::display::display_mode::DisplayMode;
use crate::feature::image::present::present_inputs::PresentInputs;
use crate::feature::scene::drawable::light_casters::LightCasters;
use crate::feature::scene::grass::grass_view_frame::GrassViewFrame;
use crate::feature::tracing::reflections::trace::reflection_trace::ReflectionTrace;
use crate::feature::tracing::sun_trace::sun_trace_feature::SunTraceFeature;
use crate::feature::view::overhead_windows::OverheadWindows;
use crate::feature::view::scene_features::SceneFeatures;
use crate::feature::weather::fog::fog_feature::FogFeature;
use crate::feature::weather::water::water_frame::WaterFrame;
use crate::feature::weather::wet::puddles::level_surface::LevelSurface;
use crate::frame::frame_encoder::FrameEncoder;
use crate::pass::frame_lighting::lighting_frame::LightingFrame;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::lights::lights_frame::LightsFrame;
use crate::scene::level::view::scene_view::SceneView;
use crate::scene::level::view::view_prepare::ViewPrepare;
use crate::scene::static_scene::draw::static_sight::StaticSight;

impl SceneView {
  /// The units the weather drives: the shadows' maps, the effects laid over the lit frame (whose flares give the sky its
  /// sun), the sky, the water, the wind, the falling rain and the wet surfaces following it, and the bloom. Answers what
  /// the lighting draws the sky with.
  pub(super) fn prepare_weather(
    &mut self,
    scene: &LevelScene,
    frame: &ViewPrepare<'_>,
    sight: &StaticSight,
    time: f32,
    now: Instant,
  ) -> LightingFrame {
    let (device, options, lighting) = (frame.device, frame.options, frame.lighting);
    let size: Option<(u32, u32)> = self.targets.as_ref().map(|targets| (targets.width, targets.height));

    self.features.shadows.prepare(
      device,
      (options.features.shadows.get_cascade_count() > 0).then_some(options.features.shadows.resolution),
    );

    let sun_sprite: Option<SunSprite> = self.stages.effects.prepare(frame, now, time);

    self.features.sky.prepare(frame, sun_sprite, &scene.environments.1);
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
    let falling: Option<RainUniform> = self.features.rain.prepare(frame, scene.gust, time, windows);

    self.features.rain.follow_display(
      DisplayMode::of(options).get_display(&options.features.grading, &self.features.exposure.get_metering()),
    );
    self.features.wet.prepare(
      frame,
      &scene.wetness,
      falling.as_ref(),
      time,
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

    let lighting_frame: LightingFrame = self
      .features
      .sky
      .get_lighting_frame(frame, &self.features.exposure, time);

    self
      .features
      .bloom
      .prepare(frame.passes, options, lighting.engine, size);

    lighting_frame
  }

  /// The scene's units: the static cull; the grass, which reads its discard, plants around last frame's puddles and
  /// casts by the trace or the cascades; the lights, which read the cull's levels of detail; then what the views off
  /// the viewport draw the scene with.
  pub(super) fn prepare_scene(
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
      .prepare(&scene.statics, options, view, frame.drawn);

    let is_traced: bool =
      options.is_light_traced() && frame.passes.tracing.scene_trace.is_traced() && scene.trace.is_some();
    // The sun's cascades shadow the frame where they are asked for and the sun is not traced in their place.
    let is_sun_traced: bool = is_traced && SunTraceFeature::is_asked(options, self.shared.resolve);
    let is_cascaded: bool = options.features.shadows.get_cascade_count() > 0 && !is_sun_traced;

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
        frame.passes.weather.wet.get_dry()[1],
        &self.features.wet,
      ),
      is_traced,
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

    let light_casters: Option<LightCasters> = self.features.grass.get_light_casters();

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
        sway: &Self::to_sway(&scene.statics, &self.features.wind),
        casters: light_casters.as_slice(),
        campfire_shares: &scene.campfire_shares,
        is_scene_first: frame.is_scene_first,
        is_physical: options.is_light_physical(),
        is_specular_linear: options.is_specular_linear(),
        anchor: (options.get_physical_light_level() == Some(RenderLightLevel::Anchored))
          .then(|| lighting.get_ambient(&options.features.light_scales)),
        is_falloff_inverse_square: options.is_falloff_inverse_square(),
        light_scale: lighting_frame.light_units.local
          * lighting_frame.light_units.light_gain
          * lighting_frame.pre_exposure.current,
        time,
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

  /// The view's own units: the reflections, the view's trace, the composited surfaces in `sight`, the scene's stage and
  /// the image's.
  pub(super) fn prepare_view(
    &mut self,
    scene: &LevelScene,
    encoder: &mut FrameEncoder,
    frame: &ViewPrepare<'_>,
    sight: &StaticSight,
  ) {
    self
      .features
      .reflections
      .prepare(frame, self.shared.resolve, &self.features.wet);
    self.features.view_trace.prepare(
      frame,
      &mut *encoder,
      scene.trace.as_ref(),
      self.features.grass.get_cards_blas(),
    );
    self.features.composited.prepare(frame, scene, &mut *encoder, sight);
    self.stages.scene.prepare(frame.options, frame.view);
    self.stages.image.prepare(frame);
  }

  /// The lighting the passes read, the enhanced fog's height fog faded out indoors; the lighting's stage, which reads
  /// it; then the particles and the present.
  pub(super) fn prepare_lighting(
    &mut self,
    scene: &mut LevelScene,
    encoder: &mut FrameEncoder,
    frame: &ViewPrepare<'_>,
    lighting_frame: &LightingFrame,
    time: f32,
  ) {
    let (device, options, view) = (frame.device, frame.options, frame.view);

    self.shared.lighting = LightingUniform::new(frame.lighting, view.view, options, lighting_frame);
    self.shared.lighting.height_fog.w *= FogFeature::get_indoor_share(scene.camera_hemi);
    self.stages.lighting.prepare(frame, &self.shared);

    scene.particles.fill(view, options, &mut self.features.particles);

    self.features.particles.upload(device, &mut *encoder);
    self.prepare_present(options, frame.output, time, self.shared.resolve.noise_frame);
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

  /// Notes what the window pass reads, once every other unit knows what it draws: whether the water, a particle, the
  /// enhanced rain's splashes or a distorting surface of the scene moves what is seen through it, which bloom it lays
  /// over the frame, the sky debanding, its noise turning with `time`, and the film's grain, drawn for `noise_frame`.
  fn prepare_present(&mut self, options: &RenderViewOptions, output: RenderRect, time: f32, noise_frame: u32) {
    if self.targets.is_none() {
      return;
    }

    let water: &RenderWaterSettings = &options.features.water;
    // The enhanced water refracts what is behind it itself and writes no distortion.
    let is_water_distorting: bool =
      water.is_enabled && water.is_distorted && water.mode != RenderWaterMode::Enhanced && options.mode.is_lit;
    let features: &SceneFeatures = &self.features;
    let inputs: PresentInputs = PresentInputs {
      is_upscaled: features.resolve.is_rescaled(),
      is_occluded: self.stages.lighting.is_occluded(),
      is_indirect: self.stages.lighting.is_indirect(),
      reflection_ratio: features.reflections.get_trace().map(ReflectionTrace::get_ratio),
      is_distorting: !options.mode.is_wireframe
        && (is_water_distorting
          || features.particles.is_distorting()
          || features.rain.is_distorting()
          || features.composited.is_distorting()),
      selection_color: features.overlays.get_selection_color(),
      marks: features.helpers.get_present_marks(),
      is_engine_bloomed: features.bloom.is_engine_drawn(),
      is_enhanced_bloomed: features.bloom.is_enhanced_drawn(),
      grass_reach: features.grass.get_cascade_reach(),
      light: self.shared.lighting.get_light_encoding(),
      metering: features.exposure.get_metering(),
      grain_frame: noise_frame,
    };

    self.features.bloom.keep_for_distortion(inputs.is_distorting);
    self.features.present.prepare(options, output, inputs, time);
  }
}
