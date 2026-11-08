use glam::Vec4;
use xrf_renderer_core::{
  FrameGraph, GraphBuffer, GraphColorAttachment, GraphDeclaration, GraphTexture, StorageValue, UniformBinding,
};

use crate::contract::feature::lighting::model::render_light_level::RenderLightLevel;
use crate::contract::feature::tracing::render_reflection_probe::RenderReflectionProbe;
use crate::contract::report::applied::render_applied_report::RenderAppliedReport;
use crate::feature::effects::flares::sun_sprite::SunSprite;
use crate::feature::tracing::surroundings::surroundings_cube::SurroundingsCube;
use crate::feature::view::view_feature::ViewFeature;
use crate::feature::weather::sky::deband::sky_deband::SkyDeband;
use crate::feature::weather::sky::dome::sky_haze_parameters::SkyHazeParameters;
use crate::feature::weather::sky::dome::sky_motion_parameters::SkyMotionParameters;
use crate::feature::weather::sky::irradiance::sky_harmonics::SkyHarmonics;
use crate::feature::weather::sky::irradiance::sky_irradiance::SkyIrradiance;
use crate::feature::weather::sky::prefilter::sky_prefilter::SkyPrefilter;
use crate::feature::weather::sky::prefilter::sky_prefilter_parameters::SkyPrefilterParameters;
use crate::feature::weather::sky::prefilter::sky_prefilter_pass::SkyPrefilterPass;
use crate::feature::weather::sky::prefilter::sky_prefilter_uniform::SkyPrefilterUniform;
use crate::feature::weather::sky::sky_parameters::SkyParameters;
use crate::feature::weather::sky::sky_views::SkyViews;
use crate::frame::periodic::periodic_job::PeriodicJob;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic::periodic_ticket::PeriodicTicket;
use crate::frame::targets::view_target_handles::ViewTargetHandles;
use crate::lighting::unit::daylight::Daylight;
use crate::lighting::unit::light_units::LightUnits;
use crate::lighting::weather::render_sky::RenderSky;
use crate::pass::frame_lighting::lighting_frame::LightingFrame;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::view::view_binding::ViewBinding;
use crate::scene::level::level_time::LevelTime;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::level::view::view_frame::ViewFrame;
use crate::scene::level::view::view_prepare::ViewPrepare;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// The sky a view draws and reflects: its textures as the weather holds them this frame, with the sun's sprite, the
/// prefiltered cube the reflections and the enhanced specular read, filled again as the frame's scheduler grants it (at
/// once when new, else after `REFILL_WAIT_LEAST` frames and within `REFILL_WAIT_MOST` more, as the sky and its light
/// move slowly), the haze map the distance fades into, and under the physical light the sky's irradiance.
#[derive(Default)]
pub struct SkyFeature {
  views: Option<SkyViews>,
  /// The sun's sprite as the sky draws it, where one is drawn.
  sun_sprite: Option<SunSprite>,
  /// The sky's textures smoothed where they band, which the combine draws the sky with while the debanding is enhanced.
  deband: Option<SkyDeband>,
  /// The prefiltered cube, while the enhanced reflections and specular read it.
  prefilter: Option<SkyPrefilter>,
  /// What the sky group binds in the prefiltered cube's place while none is kept: the weather's placeholder cube.
  unfiltered: Option<wgpu::TextureView>,
  /// Whether the haze map is drawn this frame.
  is_hazing: bool,
  /// The sky's irradiance, while the light is physical.
  irradiance: Option<SkyIrradiance>,
  /// The harmonics this frame lights the hemisphere by: the last the irradiance read back, none under the engine's
  /// light.
  harmonics: SkyHarmonics,
  /// Whether the prefiltered cube was filled since it was made.
  is_filled: bool,
  /// The refill offered this frame, whether this frame fills the cube, frames a refill has waited, and frames since
  /// the last fill before the next is offered.
  refill: Option<PeriodicTicket>,
  is_refilled: bool,
  waited: u32,
  rested: u32,
}

impl SkyFeature {
  /// Frames a filled cube waits before a refill is offered, so spare budget does not refill a slow sky every frame.
  const REFILL_WAIT_LEAST: u32 = 4;
  /// Frames an offered refill of the prefiltered cube waits at most.
  const REFILL_WAIT_MOST: u32 = 4;
  /// Milliseconds a refill costs before the timer measured one.
  const REFILL_SEED: f32 = 0.2;
  /// The clouds' two layers' drift over their textures a second, `xy` the first's and `zw` the second's: each layer's
  /// way, `setHP(PI_DIV_4, 0)` and `setHP(PI_DIV_4 + PI_DIV_8, 0)` in `x` and `z` through the byte `RenderClouds` packs
  /// a direction into, times its speed, `CLOUD_SPEED0` and `CLOUD_SPEED1` (`shared/cloudconfig.h`), on `timers.z`
  /// run a tenth as fast.
  const CLOUD_DRIFT: [f64; 4] = [
    -0.709_803_9 * 0.1 * 0.1,
    0.701_960_8 * 0.1 * 0.1,
    -0.929_411_8 * 0.05 * 0.1,
    0.380_392_2 * 0.05 * 0.1,
  ];

  /// How far the clouds' layers have drifted over their textures at `time`, each within the textures' repeat of one,
  /// the first's in `xy` and the second's in `zw`.
  pub fn get_cloud_drift(time: LevelTime) -> Vec4 {
    Vec4::from_array(Self::CLOUD_DRIFT.map(|rate| time.get_turns(rate)))
  }

  /// Takes this frame's sky: its textures and the sun's sprite (`sun_sprite`) as the weather textures hold them, the
  /// level's environment cubes beside them, and the prefiltered cube at the size the settings ask, made again where
  /// that changed.
  pub fn prepare(
    &mut self,
    &ViewPrepare {
      device,
      passes,
      options,
      lighting,
      weather_textures,
      ..
    }: &ViewPrepare<'_>,
    sun_sprite: Option<SunSprite>,
    environments: &[String],
  ) {
    let sky: &RenderSky = &lighting.sky;
    self.views = Some(passes.sky.collect(
      weather_textures,
      sky,
      sun_sprite.as_ref().map(|sprite| sprite.texture.as_str()),
      environments,
    ));
    self.sun_sprite = sun_sprite;
    self.is_hazing = options.mode.is_lit && options.show.is_sky_visible && options.show.is_sky_hazed;

    if SkyDeband::is_asked(options) {
      self.deband.get_or_insert_with(SkyDeband::default).prepare(
        device,
        weather_textures,
        sky,
        options.show.is_clouded,
      );
    } else {
      self.deband = None;
    }

    // The physical light lights the hemisphere by the sky's irradiance; the engine's by its cubes.
    if options.is_light_physical() {
      self.harmonics = self
        .irradiance
        .get_or_insert_with(|| SkyIrradiance::new(device))
        .take_harmonics();
    } else {
      self.irradiance = None;
      self.harmonics = SkyHarmonics::default();
    }

    // The prefiltered cube is the enhanced reflections' and specular's alone.
    self.refill = None;
    self.is_refilled = false;

    if !options.features.reflections.is_drawn() {
      self.prefilter = None;
      self.is_filled = false;
      self.unfiltered = Some(weather_textures.get_view(None, WeatherTextureKind::Cube).clone());

      return;
    }

    let probe: RenderReflectionProbe = options.features.reflections.probe;

    if self.prefilter.as_ref().is_some_and(|it| it.probe != probe) {
      self.prefilter = None;
      self.is_filled = false;
    }

    self.prefilter.get_or_insert_with(|| SkyPrefilter::new(device, probe));
    self.unfiltered = None;
  }

  /// Offers `scheduler` a refill of the prefiltered cube, while one is kept.
  pub fn offer_refill(&mut self, scheduler: &mut PeriodicScheduler) {
    if self.prefilter.is_none() {
      return;
    }

    if self.is_filled && self.rested < Self::REFILL_WAIT_LEAST {
      self.rested += 1;

      return;
    }

    let job: PeriodicJob = if self.is_filled {
      PeriodicJob::new_waiting(
        &SkyPrefilterPass::PASSES,
        Self::REFILL_SEED,
        1.0,
        self.waited,
        Self::REFILL_WAIT_MOST,
      )
    } else {
      PeriodicJob::new_now(&SkyPrefilterPass::PASSES, Self::REFILL_SEED, 1.0)
    };

    self.refill = Some(scheduler.offer(job));
  }

  /// Takes the refill `scheduler` granted; one not granted waits a frame more.
  pub fn commit_refill(&mut self, scheduler: &PeriodicScheduler) {
    let Some(ticket) = self.refill.take() else {
      return;
    };

    self.is_refilled = scheduler.is_granted(ticket);

    if self.is_refilled {
      self.is_filled = true;
      self.waited = 0;
      self.rested = 0;
    } else {
      self.waited += 1;
    }
  }

  /// Forgets a first fill this frame made, where its frame was not submitted: it fills again at once; and drops the
  /// harmonics that frame copied out.
  pub fn abandon_frame(&mut self) {
    if self.is_refilled {
      self.is_filled = false;
    }

    if let Some(irradiance) = &self.irradiance {
      irradiance.abandon();
    }

    if let Some(deband) = &mut self.deband {
      deband.abandon_frame();
    }
  }

  /// Asks for the harmonics copied out with the frame just submitted.
  pub fn request_readback(&self) {
    if let Some(irradiance) = &self.irradiance {
      irradiance.request();
    }
  }

  /// Writes into `applied` whether the sky is drawn with its textures smoothed where they band.
  pub fn describe(&self, applied: &mut RenderAppliedReport) {
    applied.is_sky_debanded = self.deband.is_some();
  }

  /// The sun's sprite as the sky draws it, where one is drawn.
  pub fn get_sun_sprite(&self) -> Option<&SunSprite> {
    self.sun_sprite.as_ref()
  }

  /// Writes the sky's motion where nothing was drawn, which the resolves carry its history by.
  pub fn add_motion_pass<'a>(
    &self,
    graph: &mut FrameGraph<'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    handles: ViewTargetHandles,
  ) {
    let sky_motion: SkyMotionParameters = SkyMotionParameters {
      depth_target: handles.depth,
    };

    graph
      .add_raster_pass("sky motion")
      .parameters(&sky_motion)
      .color(GraphColorAttachment::new(handles.motion, wgpu::LoadOp::Load))
      .record(move |context| passes.weather.sky_motion.record(context, view, &sky_motion));
  }

  /// What the lighting draws the sky with this frame: a keyframe whose textures are not all up blended out, so a sky
  /// still going up shows the other one; whether both irradiance cubes are up; the clouds' drift at `time`; the sun's sprite
  /// while its texture is up; the exposure and the display the view decided (`shared`); at the physical light levels the
  /// world's light, and the units the light is in by the sky last projected.
  pub fn get_lighting_frame(
    &self,
    &ViewPrepare {
      options,
      lighting,
      weather_textures,
      ..
    }: &ViewPrepare<'_>,
    shared: &ViewFrame,
    time: LevelTime,
  ) -> LightingFrame {
    let sky: &RenderSky = &lighting.sky;
    let daylight: Option<Daylight> = (options.is_light_physical()
      && options.features.light_level == RenderLightLevel::Physical)
      .then(|| Daylight::of(lighting, options.features.sun_tint));
    let side = |index: usize| {
      let clouds: Option<&str> = sky.clouds.textures[index]
        .as_deref()
        .filter(|_| options.show.is_clouded);

      weather_textures.is_settled(sky.textures[index].as_deref())
        && weather_textures.is_settled(sky.environments[index].as_deref())
        && weather_textures.is_settled(clouds)
    };

    LightingFrame {
      exposure: shared.exposure,
      display: shared.display,
      light_units: LightUnits::of(options, lighting, &self.harmonics, daylight.as_ref()),
      daylight,
      sky_blend: match (side(0), side(1)) {
        (true, false) => 0.0,
        (false, true) => 1.0,
        _ => sky.blend,
      },
      is_irradiance_up: sky
        .environments
        .iter()
        .all(|reference| reference.as_deref().is_some_and(|it| weather_textures.is_up(it))),
      cloud_drift: Self::get_cloud_drift(time),
      sky_harmonics: self.harmonics,
      sun_sprite: self
        .get_sun_sprite()
        .filter(|sprite| weather_textures.is_up(&sprite.texture))
        .map_or(Vec4::ZERO, |sprite| sprite.look),
    }
  }

  /// The sky's textures this frame, once prepared.
  pub fn get_views(&self) -> Option<&SkyViews> {
    self.views.as_ref()
  }

  /// Whether the haze map is drawn this frame.
  pub fn is_hazing(&self) -> bool {
    self.is_hazing && self.views.is_some()
  }

  /// Adds the sky: its textures and the prefiltered cube imported, the surroundings' face drawn into its first mip
  /// where `surroundings` draws one, the cube filled where this frame refills it, and the haze map drawn; answers the
  /// sky's parameters, which the combine, the water and the composited surfaces bind, none before it is prepared.
  pub fn add_passes<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    surroundings: Option<&'a SurroundingsCube>,
  ) -> Option<SkyParameters<'a>> {
    let ViewRecord {
      passes, handles, lit, ..
    } = record;
    let views: &'a SkyViews = self.views.as_ref()?;
    let cube: &'a wgpu::TextureView = match (&self.prefilter, &self.unfiltered) {
      (Some(prefilter), _) => &prefilter.cube,
      (None, Some(unfiltered)) => unfiltered,
      (None, None) => return None,
    };
    let sky: SkyParameters<'_> = passes.sky.import(views, cube, &mut *graph, &mut *bindings);

    if let Some(irradiance) = &self.irradiance {
      irradiance.add_passes(GraphDeclaration::new(graph, bindings, runtime), passes, &sky, &lit);
    }

    if let Some(prefilter) = &self.prefilter {
      self.add_prefilter_passes(
        GraphDeclaration::new(graph, bindings, runtime),
        record,
        &sky,
        prefilter,
        surroundings,
      );
    }

    if self.is_hazing {
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
        .record(move |context| passes.weather.sky_haze.record(context, &parameters, &sky));
    }

    Some(sky)
  }

  /// The sky's parameters as the combine draws the sky, from those `add_passes` answered (`sky`): with the enhanced
  /// debanding, its textures' smoothed copies in their place, each smoothed first where this frame copies it; every
  /// other reader of the sky keeps the textures as they are.
  pub fn add_drawn_sky<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    sky: SkyParameters<'a>,
  ) -> SkyParameters<'a> {
    match &self.deband {
      Some(deband) => deband.add_passes(graph, bindings, passes, sky),
      None => sky,
    }
  }

  /// Adds the passes filling the prefiltered sky cube this frame, a mip each: the first the surroundings cube's face
  /// drawn this frame, the level around the camera lit with the sky behind, or the sky's cubes alone as the sky is
  /// drawn where none is; each after it from the mips before it over its roughness's lobe.
  fn add_prefilter_passes<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    sky: &SkyParameters<'a>,
    prefilter: &'a SkyPrefilter,
    surroundings: Option<&'a SurroundingsCube>,
  ) {
    let ViewRecord { passes, lit, .. } = record;
    let (lighting, exposure): (UniformBinding<LightingUniform>, GraphBuffer) = (lit.lighting, lit.exposure);
    let empty: GraphTexture = bindings.import_view(
      &mut *graph,
      "sky prefilter none",
      passes.weather.sky_prefilter.get_empty(),
    );

    for (index, level) in prefilter.levels.iter().enumerate() {
      let source: GraphTexture = level.source.as_ref().map_or(empty, |before| {
        bindings.import_view(&mut *graph, "sky prefilter before", before)
      });
      let faces: GraphTexture = bindings.import_view(&mut *graph, "sky prefilter level", &level.target);

      if let (0, Some(cube)) = (index, surroundings) {
        cube.add_passes(GraphDeclaration::new(graph, bindings, runtime), record, sky, faces);

        continue;
      }

      if !self.is_refilled {
        continue;
      }
      let settings: UniformBinding<SkyPrefilterUniform> = runtime.push_uniform(&SkyPrefilterUniform::new(
        index as u32,
        prefilter.levels.len() as u32,
        level.size,
        prefilter.probe.get_size(),
        SkyPrefilterPass::SAMPLES,
      ));
      let parameters: SkyPrefilterParameters<'_> = SkyPrefilterParameters {
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
        .record(move |context| passes.weather.sky_prefilter.record(context, size, &parameters));
    }
  }
}

impl ViewFeature for SkyFeature {
  fn release(&mut self) {
    self.prefilter = None;
    self.irradiance = None;
    self.deband = None;
  }
}
