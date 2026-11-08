use xrf_renderer_core::{GraphDeclaration, GraphTexture};

use crate::context::gpu_requirements::can_trace;
use crate::contract::feature::tracing::render_reflection_settings::RenderReflectionSettings;
use crate::contract::feature::weather::water::render_water_mode::RenderWaterMode;
use crate::contract::feature::weather::water::render_water_settings::RenderWaterSettings;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::effects::rain::rings::drop_rings::DropRings;
use crate::feature::view::view_feature::ViewFeature;
use crate::feature::view::view_frame_end::ViewFrameEnd;
use crate::feature::view::view_restart::ViewRestart;
use crate::feature::weather::sky::sky_views::SkyViews;
use crate::feature::weather::water::draw::traced_water::TracedWater;
use crate::feature::weather::water::draw::water_draw::WaterDraw;
use crate::feature::weather::water::enhanced::enhanced_water_frame::EnhancedWaterFrame;
use crate::feature::weather::water::enhanced::enhanced_water_scroll::EnhancedWaterScroll;
use crate::feature::weather::water::enhanced::enhanced_water_uniform::EnhancedWaterUniform;
use crate::feature::weather::water::enhanced::water_flow::WaterFlow;
use crate::feature::weather::water::enhanced::water_weather::WaterWeather;
use crate::feature::weather::water::reflection::water_reflection::WaterReflection;
use crate::feature::weather::water::reflection::water_reflection_trace::WaterReflectionTrace;
use crate::feature::weather::water::water_frame::WaterFrame;
use crate::feature::weather::water::water_inputs::WaterInputs;
use crate::feature::weather::water::water_uniform::WaterUniform;
use crate::frame::targets::screen_tiles::ScreenTiles;
use crate::pass::level_passes::LevelPasses;
use crate::scene::level::view::record::view_record::ViewRecord;

/// A view's water as it lasts between frames: its settings and what they make its uniforms this frame, how far the
/// enhanced water's maps have scrolled, and its reflection histories, kept only while it is asked to reflect.
#[derive(Default)]
pub(crate) struct WaterFeature {
  settings: RenderWaterSettings,
  /// Whether this frame draws the water at all: it is on, some is in sight, and the view is not wireframe.
  is_drawn: bool,
  /// The tiles of the frame the water in sight may touch, which its reflection's blur keeps to.
  tiles: ScreenTiles,
  uniform: WaterUniform,
  enhanced: EnhancedWaterUniform,
  reflection: Option<WaterReflection>,
  /// How far the enhanced water's maps have scrolled.
  flow: WaterFlow,
  /// Frames prepared, which turns which of its pixels trace.
  frame: u32,
  /// Whether this frame's enhanced water traces every pixel's reflected ray.
  is_fully_traced: bool,
}

impl WaterFeature {
  /// Draws the water of a frame's `record` where it draws, from what it reads beside it (`inputs`): the enhanced water
  /// rippled by the enhanced drops' rings and shone on by the sun where it reaches the bottom, its rays the screen left
  /// traced where the scene's trace is built. Answers the G-buffer's depth with the nearest water's over it where the
  /// water draws.
  pub(crate) fn add_passes<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    WaterInputs {
      views,
      before_water,
      args,
      guides,
      rings,
      sun_seen,
      sky,
      traced,
    }: WaterInputs<'a>,
  ) -> Option<GraphTexture> {
    let traced: Option<TracedWater<'a>> = sky.zip(traced).map(|(sky, scene)| TracedWater { sky, scene });
    let views: &SkyViews = views.filter(|_| self.is_drawn())?;
    let passes: LevelPasses<'a> = record.passes;
    let draw: WaterDraw<'a> = WaterDraw {
      targets: record.handles,
      before_water: before_water.filter(|_| self.is_enhanced()),
      view: record.view,
      textures: record.textures,
      layouts: record.statics.get_camera_draws().layouts,
      args,
      lit: record.lit,
      skies: [&views.cubes[0], &views.cubes[1]],
      sky_sampler: passes.sky.get_clamp(),
      water: self,
      depth_pyramid: &passes.scene.pyramid,
      scene_trace: &passes.tracing.scene_trace,
      traced,
      rings,
      sun_seen,
      guides,
    };

    Some(passes.weather.water.add_passes(graph, bindings, runtime, draw))
  }

  /// Takes this frame's settings: makes or drops the reflection's histories, and works out the uniforms.
  pub(crate) fn prepare(&mut self, device: &wgpu::Device, options: &RenderViewOptions, frame: WaterFrame<'_>) {
    let settings: RenderWaterSettings = options.features.water;

    self.settings = settings;
    self.is_drawn = settings.is_enabled && !frame.tiles.is_empty() && !options.mode.is_wireframe;
    self.tiles = frame.tiles;
    // The wave's motion runs from where it stood the last frame; still on a first frame.
    self.uniform = WaterUniform::new(&settings, frame.intensity, frame.time.now).following(frame.time.get_previous());
    // Advanced while the engine's water draws too, so switching back does not lurch the maps by the time between.
    self.flow.advance(
      frame.time,
      &settings.enhanced,
      WaterWeather {
        wind: frame.wind,
        rain: frame.rain,
      },
    );
    self.is_fully_traced = false;

    // Kept while the enhanced water is asked for, though none is in sight, so water coming back into view takes no new
    // histories; a frame drawing none forgets the history in them (`end_frame`).
    if !settings.is_enabled || settings.mode != RenderWaterMode::Enhanced || options.mode.is_wireframe {
      self.reflection = None;

      return;
    }

    let reflections: RenderReflectionSettings = options.features.reflections;

    match frame.targets {
      Some(targets) if settings.enhanced.reflectivity > 0.0 && reflections.is_water_reflected => {
        if self
          .reflection
          .as_ref()
          .is_none_or(|reflection| reflection.size != (targets.width, targets.height))
        {
          self.reflection = Some(WaterReflection::new(device, targets));
        }
      }
      _ => self.reflection = None,
    }

    let is_traced: bool = reflections.is_traced && can_trace(device);
    // Every pixel tracing every frame leaves a record at its place, at a turn of one, never waiting dim.
    let is_fully_traced: bool = is_traced && reflections.is_water_fully_traced;

    self.frame = self.frame.wrapping_add(1);
    self.is_fully_traced = is_fully_traced && self.is_drawn && self.reflection.is_some();

    if let Some(reflection) = &mut self.reflection {
      reflection.fit_trace(device, is_traced, is_fully_traced);
    }

    self.enhanced = EnhancedWaterUniform::new(
      &settings.enhanced,
      EnhancedWaterFrame {
        wind: frame.wind,
        rain: frame.rain,
        scroll: EnhancedWaterScroll::new(&self.flow, settings.enhanced.parallax_height > 0.0),
        is_reflected: self.reflection.is_some(),
        has_history: self
          .reflection
          .as_ref()
          .is_some_and(|reflection| reflection.histories.is_valid()),
        trace: WaterReflectionTrace {
          distance: if is_traced { reflections.distance } else { 0.0 },
          turns: if is_fully_traced {
            1
          } else {
            reflections.get_trace_interval()
          },
          frame: self.frame,
          dim_turns: if reflections.is_ray_budgeted && !is_fully_traced {
            WaterReflection::DIM_TURNS
          } else {
            1
          },
          march_steps: reflections.get_water_steps(),
          is_full: is_fully_traced,
        },
      },
    );
  }

  pub(crate) fn is_drawn(&self) -> bool {
    self.is_drawn
  }

  /// Whether this frame's enhanced water draws and traces every pixel's reflected ray, which the grass's cards stand
  /// in.
  pub(crate) fn is_fully_traced(&self) -> bool {
    self.is_fully_traced
  }

  /// Whether the enhanced water draws, reading the scene before it to refract.
  pub(crate) fn is_enhanced(&self) -> bool {
    self.is_drawn && self.settings.mode == RenderWaterMode::Enhanced
  }

  /// The tiles of the frame the water in sight may touch.
  pub(crate) fn get_tiles(&self) -> &ScreenTiles {
    &self.tiles
  }

  pub(crate) fn get_uniform(&self) -> &WaterUniform {
    &self.uniform
  }

  /// Follows the enhanced drops' rings this frame.
  pub(crate) fn follow_rings(&mut self, rings: DropRings) {
    self.enhanced.rings_tile = rings.tile;
    self.enhanced.rings_reach = rings.reach;
    self.enhanced.rings_strength = rings.strength;
  }

  /// Follows the grass's cards this frame, readied after the water: the metres from the eye within which the trace
  /// stands every tuft whole (`TraceGrassUniform::card_reach`, none where it traces none), so the screen leaves the
  /// plants there to the trace while every pixel traces, and answers them where the trace would meet nothing.
  pub(crate) fn follow_grass_cards(&mut self, card_reach: f32) {
    self.enhanced.plant_reach = if self.is_fully_traced { card_reach } else { 0.0 };
  }

  pub(crate) fn get_enhanced_uniform(&self) -> &EnhancedWaterUniform {
    &self.enhanced
  }

  /// The enhanced water's reflection, while it reflects.
  pub(crate) fn get_reflection(&self) -> Option<&WaterReflection> {
    self.reflection.as_ref()
  }
}

impl ViewFeature for WaterFeature {
  fn release(&mut self) {
    self.reflection = None;
  }

  /// The reflection this frame wrote becomes the history, or the history is forgotten where no water drew.
  fn end_frame(&mut self, _end: &mut ViewFrameEnd<'_>) {
    let Some(reflection) = self.reflection.as_mut() else {
      return;
    };

    if self.is_drawn {
      reflection.histories.swap();
    } else {
      reflection.histories.invalidate();
    }
  }

  /// Its motion restarted, its maps' scroll, its traced pixels' turn and its reflection's histories start again from
  /// nothing, its wave's last place over with the view's last frame; a cut leaves the reflection's history nothing to
  /// read, its scroll running on.
  fn restart(&mut self, restart: ViewRestart) {
    match restart {
      ViewRestart::Motion => {
        self.flow = WaterFlow::default();
        self.frame = 0;
        self.reflection = None;
      }
      ViewRestart::Cut => {
        if let Some(reflection) = &mut self.reflection {
          reflection.histories.invalidate();
        }
      }
    }
  }
}
