use glam::Vec4;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphColorAttachment, GraphRuntime, GraphTexture, StorageValue,
  UniformBinding,
};

use crate::contract::render_reflection_probe::RenderReflectionProbe;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::periodic_job::PeriodicJob;
use crate::frame::periodic_scheduler::PeriodicScheduler;
use crate::frame::periodic_ticket::PeriodicTicket;
use crate::frame::sky_prefilter::SkyPrefilter;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::lighting::render_sky::RenderSky;
use crate::pass::level_passes::LevelPasses;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::sky_haze_parameters::SkyHazeParameters;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::sky_prefilter_parameters::SkyPrefilterParameters;
use crate::pass::sky_prefilter_pass::SkyPrefilterPass;
use crate::pass::sky_prefilter_uniform::SkyPrefilterUniform;
use crate::pass::static_cull_params::StaticCullParams;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::lighting_handles::LightingHandles;
use crate::scene::level::sky_views::SkyViews;
use crate::scene::level::surroundings_cube::SurroundingsCube;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// The sky a view draws and reflects: its textures as the weather holds them this frame, with the sun's sprite, the
/// prefiltered cube the reflections and the enhanced specular read, filled again as the frame's scheduler grants it
/// (at once when new, else within `REFILL_WAIT_MOST` frames, as the sky and its light move slowly), and the haze map the
/// distance fades into.
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
  /// Whether the prefiltered cube was filled since it was made.
  is_filled: bool,
  /// The refill offered this frame, whether this frame fills the cube, and frames a refill has waited.
  refill: Option<PeriodicTicket>,
  is_refilled: bool,
  waited: u32,
}

impl SkyFeature {
  /// Frames a refill of the prefiltered cube waits at most.
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
    weather_textures: &WeatherTextureCache,
  ) {
    self.views = Some(passes.sky.collect(
      weather_textures,
      (sky, sun_sprite.as_ref().map(|(texture, _)| texture.as_str())),
      environments,
    ));
    self.sun_sprite = sun_sprite;
    self.is_hazing = options.mode.is_lit && options.show.is_sky_visible && options.show.is_sky_hazed;

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
    } else {
      self.waited += 1;
    }
  }

  /// Forgets a first fill this frame made, where its frame was not submitted: it fills again at once.
  pub fn abandon_frame(&mut self) {
    if self.is_refilled {
      self.is_filled = false;
    }
  }

  /// The sun's sprite as the sky draws it, where one is drawn.
  pub fn get_sun_sprite(&self) -> Option<&(String, Vec4)> {
    self.sun_sprite.as_ref()
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
      (&'a StaticCullParams, &'a wgpu::BindGroup),
    )>,
  ) -> Option<SkyParameters<'a>> {
    let views: &'a SkyViews = self.views.as_ref()?;
    let cube: &'a wgpu::TextureView = match (&self.prefilter, &self.unfiltered) {
      (Some(prefilter), _) => &prefilter.cube,
      (None, Some(unfiltered)) => unfiltered,
      (None, None) => return None,
    };
    let sky: SkyParameters = passes.sky.import((views, cube), &mut *graph, &mut *bindings);

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
  /// The prefiltered cube, every mip of its six faces.
  fn get_memory(&self) -> u64 {
    self.prefilter.as_ref().map_or(0, |prefilter| {
      let size: u64 = u64::from(prefilter.probe.get_size());

      (0..prefilter.levels.len() as u32)
        .map(|level| (size >> level).max(1).pow(2) * 6 * 8)
        .sum()
    })
  }

  fn release(&mut self) {
    self.prefilter = None;
  }
}
