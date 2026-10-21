use glam::Vec4;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphColorAttachment, GraphRuntime, GraphTexture, StorageValue,
  UniformBinding,
};

use crate::contract::render_light_level::RenderLightLevel;
use crate::contract::render_reflection_probe::RenderReflectionProbe;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::periodic::periodic_job::PeriodicJob;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic::periodic_ticket::PeriodicTicket;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::lighting::daylight::Daylight;
use crate::lighting::light_units::LightUnits;
use crate::lighting::render_lighting::RenderLighting;
use crate::lighting::render_sky::RenderSky;
use crate::pass::level_passes::LevelPasses;
use crate::pass::lighting_frame::LightingFrame;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::pre_exposure::PreExposure;
use crate::pass::sky_harmonics::SkyHarmonics;
use crate::pass::sky_haze_parameters::SkyHazeParameters;
use crate::pass::sky_motion_parameters::SkyMotionParameters;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::sky_prefilter_parameters::SkyPrefilterParameters;
use crate::pass::sky_prefilter_pass::SkyPrefilterPass;
use crate::pass::sky_prefilter_uniform::SkyPrefilterUniform;
use crate::pass::static_cull_uniform::StaticCullUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::sky_irradiance::SkyIrradiance;
use crate::scene::level::feature::sky_prefilter::SkyPrefilter;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::level::sky_views::SkyViews;
use crate::scene::level::surroundings_cube::SurroundingsCube;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;
use crate::scene::texture::weather_textures::WeatherTextures;

/// The sky a view draws and reflects: its textures as the weather holds them this frame, with the sun's sprite, the
/// prefiltered cube the reflections and the enhanced specular read, filled again as the frame's scheduler grants it (at
/// once when new, else after `REFILL_WAIT_LEAST` frames and within `REFILL_WAIT_MOST` more, as the sky and its light
/// move slowly), the haze map the distance fades into, and under the physical light the sky's irradiance.
#[derive(Default)]
pub struct SkyFeature {
  views: Option<SkyViews>,
  /// The sun's sprite as the sky draws it, its texture and its colour and radius, where one is drawn.
  sun_sprite: Option<(String, Vec4)>,
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

  /// Takes this frame's sky: its textures and the sun's sprite (`sun_sprite`) as the weather textures hold them, the
  /// level's environment cubes beside them, and the prefiltered cube at the size the settings ask, made again where
  /// that changed.
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    (passes, options): (LevelPasses<'_>, &RenderViewOptions),
    (sky, sun_sprite, environments): (&RenderSky, Option<(String, Vec4)>, &[String]),
    weather_textures: WeatherTextures<'_>,
  ) {
    self.views = Some(passes.sky.collect(
      weather_textures,
      (sky, sun_sprite.as_ref().map(|(texture, _)| texture.as_str())),
      environments,
    ));
    self.sun_sprite = sun_sprite;
    self.is_hazing = options.mode.is_lit && options.show.is_sky_visible && options.show.is_sky_hazed;

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
        (self.waited, Self::REFILL_WAIT_MOST),
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
  }

  /// Asks for the harmonics copied out with the frame just submitted.
  pub fn request_readback(&self) {
    if let Some(irradiance) = &self.irradiance {
      irradiance.request();
    }
  }

  /// The sun's sprite as the sky draws it, where one is drawn.
  pub fn get_sun_sprite(&self) -> Option<&(String, Vec4)> {
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
      .record(move |context| passes.sky_motion.record(context, view, &sky_motion));
  }

  /// What the lighting draws the sky with this frame: a keyframe whose textures are not all up blended out, so a sky
  /// still going up shows the other one; whether both irradiance cubes are up; the clouds' `time`; the sun's sprite
  /// while its texture is up; whether the exposure adapts (`is_adapting`) and the scale the light is written at; at the
  /// physical light levels the world's light, and the units the light is in by the sky last projected.
  pub fn get_lighting_frame(
    &self,
    (lighting, options): (&RenderLighting, &RenderViewOptions),
    weather_textures: WeatherTextures<'_>,
    (is_adapting, pre_exposure, time): (bool, PreExposure, f32),
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
      is_adapting,
      pre_exposure,
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
      clouds_time: time,
      sky_harmonics: self.harmonics,
      sun_sprite: self
        .get_sun_sprite()
        .filter(|(texture, _)| weather_textures.is_up(texture))
        .map_or(Vec4::ZERO, |(_, sprite)| *sprite),
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
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    (handles, lit): (ViewTargetHandles, LightingHandles<'a>),
    surroundings: Option<(
      &'a SurroundingsCube,
      &StaticSceneHandles,
      (&'a StaticCullUniform, &'a wgpu::BindGroup),
    )>,
  ) -> Option<SkyParameters<'a>> {
    let views: &'a SkyViews = self.views.as_ref()?;
    let cube: &'a wgpu::TextureView = match (&self.prefilter, &self.unfiltered) {
      (Some(prefilter), _) => &prefilter.cube,
      (None, Some(unfiltered)) => unfiltered,
      (None, None) => return None,
    };
    let sky: SkyParameters = passes.sky.import((views, cube), &mut *graph, &mut *bindings);

    if let Some(irradiance) = &self.irradiance {
      irradiance.add_passes((&mut *graph, &mut *bindings), passes, (&sky, &lit));
    }

    if let Some(prefilter) = &self.prefilter {
      self.add_prefilter_passes(
        (&mut *graph, &mut *bindings, runtime),
        passes,
        (&sky, prefilter, &lit),
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
        .record(move |context| passes.sky_haze.record(context, &parameters, &sky));
    }

    Some(sky)
  }

  /// Adds the passes filling the prefiltered sky cube this frame, a mip each: the first the surroundings cube's face
  /// drawn this frame, the level around the camera lit with the sky behind, or the sky's cubes alone as the sky is
  /// drawn where none is; each after it from the mips before it over its roughness's lobe.
  fn add_prefilter_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    (sky, prefilter, lit): (&SkyParameters<'a>, &'a SkyPrefilter, &LightingHandles<'a>),
    surroundings: Option<(
      &'a SurroundingsCube,
      &StaticSceneHandles,
      (&'a StaticCullUniform, &'a wgpu::BindGroup),
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

      if !self.is_refilled {
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
}

impl ViewFeature for SkyFeature {
  /// The prefiltered cube, every mip of its six faces, and the sky's irradiance.
  fn get_memory(&self) -> u64 {
    let prefilter: u64 = self.prefilter.as_ref().map_or(0, |prefilter| {
      let size: u64 = u64::from(prefilter.probe.get_size());

      (0..prefilter.levels.len() as u32)
        .map(|level| (size >> level).max(1).pow(2) * 6 * 8)
        .sum()
    });

    prefilter + self.irradiance.as_ref().map_or(0, SkyIrradiance::get_memory)
  }

  fn release(&mut self) {
    self.prefilter = None;
    self.irradiance = None;
  }
}
