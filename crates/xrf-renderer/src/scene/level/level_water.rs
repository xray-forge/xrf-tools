use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_water_mode::RenderWaterMode;
use crate::contract::render_water_settings::RenderWaterSettings;
use crate::frame::view_targets::ViewTargets;
use crate::frame::water_reflection::WaterReflection;
use crate::lighting::render_wind::RenderWind;
use crate::pass::enhanced_water_uniform::EnhancedWaterUniform;
use crate::pass::water_uniform::WaterUniform;
use crate::scene::level::water_flow::WaterFlow;

/// What a level view's water reads of its frame beside its own state: the targets, the weather's
/// `water_intensity`, wind and rain density, and the clock.
pub struct WaterFrame<'a> {
  pub targets: Option<&'a ViewTargets>,
  pub intensity: f32,
  pub wind: RenderWind,
  pub rain: f32,
  pub time: f32,
}

/// A view's water as it lasts between frames: its settings and what they make its uniforms this frame, how far the
/// enhanced water's maps have scrolled, and its reflection histories, kept only while it reflects.
#[derive(Default)]
pub struct LevelWater {
  settings: RenderWaterSettings,
  /// Whether this frame draws the water at all: it is on, and the view is not wireframe.
  is_drawn: bool,
  uniform: WaterUniform,
  enhanced: EnhancedWaterUniform,
  reflection: Option<WaterReflection>,
  /// How far the enhanced water's maps have scrolled.
  flow: WaterFlow,
}

impl LevelWater {
  /// Takes this frame's settings: makes or drops the reflection's histories, and works out the uniforms.
  pub fn prepare(&mut self, device: &wgpu::Device, options: &RenderViewOptions, frame: WaterFrame<'_>) {
    let settings: RenderWaterSettings = options.features.water;

    self.settings = settings;
    self.is_drawn = settings.is_enabled && !options.mode.is_wireframe;
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

    self.enhanced = EnhancedWaterUniform::new(
      &settings.enhanced,
      (frame.wind, frame.rain),
      self.flow,
      self
        .reflection
        .as_ref()
        .map_or((false, false), |reflection| (true, reflection.is_valid)),
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

  /// Makes the reflection a frame drew the one the next frame keeps, where the frame drew the water.
  pub fn finish_frame(&mut self) {
    if let Some(reflection) = self.reflection.as_mut().filter(|_| self.is_drawn) {
      reflection.swap();
    }
  }
}
