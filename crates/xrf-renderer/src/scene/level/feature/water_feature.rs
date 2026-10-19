use xrf_renderer_core::{FrameGraph, GraphBindings, GraphBuffer, GraphRuntime, GraphTexture};

use crate::context::gpu_requirements::can_trace;
use crate::contract::render_reflection_settings::RenderReflectionSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_water_mode::RenderWaterMode;
use crate::contract::render_water_settings::RenderWaterSettings;
use crate::pass::enhanced_water_uniform::EnhancedWaterUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::water_draw::WaterDraw;
use crate::pass::water_uniform::WaterUniform;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::feature::view_frame_end::ViewFrameEnd;
use crate::scene::level::feature::water_frame::WaterFrame;
use crate::scene::level::feature::water_reflection::WaterReflection;
use crate::scene::level::sky_views::SkyViews;
use crate::scene::level::view_record::ViewRecord;
use crate::scene::level::water_flow::WaterFlow;

/// A view's water as it lasts between frames: its settings and what they make its uniforms this frame, how far the
/// enhanced water's maps have scrolled, and its reflection histories, kept only while it reflects.
#[derive(Default)]
pub struct WaterFeature {
  settings: RenderWaterSettings,
  /// Whether this frame draws the water at all: it is on, some is in sight, and the view is not wireframe.
  is_drawn: bool,
  uniform: WaterUniform,
  enhanced: EnhancedWaterUniform,
  reflection: Option<WaterReflection>,
  /// How far the enhanced water's maps have scrolled.
  flow: WaterFlow,
  /// Frames prepared, which turns which of its pixels trace.
  frame: u32,
}

impl WaterFeature {
  /// Draws the water of a frame's `record` where it draws, over the sky's `views` and the frame copied before it
  /// (`before_water`, which the enhanced water refracts and reflects), from the cull's argument buffers `args`; its
  /// rays the screen left traced where the scene's trace is built (`traced`, with the sky's group), the enhanced water
  /// rippled by the enhanced drops' `rings`. Answers the G-buffer's depth with the nearest water's over it where the
  /// water draws.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    record: ViewRecord<'a>,
    (views, before_water, args, rings): (
      Option<&'a SkyViews>,
      Option<GraphTexture>,
      Vec<GraphBuffer>,
      GraphTexture,
    ),
    traced: Option<(SkyParameters<'a>, TraceParameters<'a>, &'a wgpu::BindGroup)>,
  ) -> Option<GraphTexture> {
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
      scene_trace: passes.scene_trace,
      traced,
      rings,
    };

    Some(passes.water.add_passes(graph, bindings, runtime, draw))
  }

  /// Takes this frame's settings: makes or drops the reflection's histories, and works out the uniforms.
  pub fn prepare(&mut self, device: &wgpu::Device, options: &RenderViewOptions, frame: WaterFrame<'_>) {
    let settings: RenderWaterSettings = options.features.water;

    self.settings = settings;
    self.is_drawn = settings.is_enabled && frame.is_in_view && !options.mode.is_wireframe;
    self.uniform = WaterUniform::new(&settings, frame.intensity, frame.time);
    // Advanced while the engine's water draws too, so switching back does not lurch the maps by the time between.
    self.flow.advance(frame.time, &settings.enhanced, frame.wind);

    if !self.is_enhanced() {
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

    self.frame = self.frame.wrapping_add(1);

    if let Some(reflection) = &mut self.reflection {
      let (width, height) = reflection.size;

      reflection.requests.fit(
        device,
        if is_traced {
          (width * height).div_ceil(WaterReflection::REQUEST_SHARE)
        } else {
          0
        },
      );
    }

    self.enhanced = EnhancedWaterUniform::new(
      &settings.enhanced,
      (frame.wind, frame.rain),
      self.flow,
      self
        .reflection
        .as_ref()
        .map_or((false, false), |reflection| (true, reflection.histories.is_valid())),
      (
        if is_traced { reflections.distance } else { 0.0 },
        reflections.get_trace_interval(),
        self.frame,
        if reflections.is_ray_budgeted {
          WaterReflection::DIM_TURNS
        } else {
          1
        },
        reflections.get_water_steps(),
      ),
    );
  }

  pub fn is_drawn(&self) -> bool {
    self.is_drawn
  }

  /// Whether the enhanced water draws, reading the scene before it to refract.
  pub fn is_enhanced(&self) -> bool {
    self.is_drawn && self.settings.mode == RenderWaterMode::Enhanced
  }

  pub fn get_uniform(&self) -> &WaterUniform {
    &self.uniform
  }

  /// Follows the enhanced drops' rings this frame: the tile they wrap over (`RainUniform::get_rings`), none without
  /// them, the metres they reach and their strength.
  pub fn follow_rings(&mut self, (tile, reach, strength): (f32, f32, f32)) {
    self.enhanced.rings_tile = tile;
    self.enhanced.rings_reach = reach;
    self.enhanced.rings_strength = strength;
  }

  pub fn get_enhanced_uniform(&self) -> &EnhancedWaterUniform {
    &self.enhanced
  }

  /// The enhanced water's reflection, while it reflects.
  pub fn get_reflection(&self) -> Option<&WaterReflection> {
    self.reflection.as_ref()
  }
}

impl ViewFeature for WaterFeature {
  /// The enhanced water's reflection histories and request list, while it reflects.
  fn get_memory(&self) -> u64 {
    self.reflection.as_ref().map_or(0, WaterReflection::get_memory)
  }

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
}
