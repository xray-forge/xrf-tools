use crate::context::gpu_requirements::can_trace;
use crate::contract::render_reflection_settings::RenderReflectionSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_water_mode::RenderWaterMode;
use crate::contract::render_water_settings::RenderWaterSettings;
use crate::frame::water_reflection::WaterReflection;
use crate::pass::enhanced_water_uniform::EnhancedWaterUniform;
use crate::pass::water_uniform::WaterUniform;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::feature::water_frame::WaterFrame;
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

    match frame.targets {
      Some(targets) if settings.enhanced.reflectivity > 0.0 => {
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

    let reflections: RenderReflectionSettings = options.features.reflections;
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
        reflections
          .trace_quality
          .get_turns()
          .max(WaterReflection::TRACE_TURNS_LEAST),
        self.frame,
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

  pub fn get_enhanced_uniform(&self) -> &EnhancedWaterUniform {
    &self.enhanced
  }

  /// The enhanced water's reflection, while it reflects.
  pub fn get_reflection(&self) -> Option<&WaterReflection> {
    self.reflection.as_ref()
  }

  /// Makes the reflection a frame drew the one the next frame keeps, where the frame drew the water; a frame that drew
  /// none leaves nothing to keep, as a frame showing no water clears its history.
  pub fn finish_frame(&mut self) {
    if let Some(reflection) = self.reflection.as_mut() {
      if self.is_drawn {
        reflection.histories.swap();
      } else {
        reflection.histories.invalidate();
      }
    }
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
}
