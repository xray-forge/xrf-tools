use crate::contract::render_view_options::RenderViewOptions;
use crate::contract::render_water_mode::RenderWaterMode;
use crate::contract::render_water_settings::RenderWaterSettings;
use crate::frame::view_targets::ViewTargets;
use crate::frame::water_reflection::WaterReflection;
use crate::lighting::render_wind::RenderWind;
use crate::pass::water_uniform::WaterUniform;
use crate::scene::level::water_flow::WaterFlow;

/// What a level view's water reads of its frame beside its own state: the targets and their epoch, the weather's
/// `water_intensity`, wind and rain density, and the clock.
pub struct WaterFrame<'a> {
  pub targets: Option<(&'a ViewTargets, u64)>,
  pub intensity: f32,
  pub wind: RenderWind,
  pub rain: f32,
  pub time: f32,
}

/// A view's water as it lasts between frames: its settings and what they make its uniform this frame, how far its maps
/// have scrolled, and the enhanced water's reflection histories, kept only while it reflects.
#[derive(Default)]
pub struct LevelWater {
  settings: RenderWaterSettings,
  /// Whether this frame draws the water at all: it is on, and the view is not wireframe.
  is_drawn: bool,
  uniform: WaterUniform,
  reflection: Option<WaterReflection>,
  /// How far the enhanced water's maps have scrolled.
  flow: WaterFlow,
}

impl LevelWater {
  /// Takes this frame's settings: makes or drops the reflection's histories, and works out the uniform.
  pub fn prepare(&mut self, device: &wgpu::Device, options: &RenderViewOptions, frame: WaterFrame<'_>) {
    let settings: RenderWaterSettings = options.features.water;

    self.settings = settings;
    self.is_drawn = settings.is_enabled && !options.mode.is_wireframe;
    self.flow.advance(frame.time, &settings, frame.wind);

    let is_reflecting: bool = self.is_refracting() && settings.reflectivity > 0.0;

    match frame.targets {
      Some((targets, epoch)) if is_reflecting => {
        if self
          .reflection
          .as_ref()
          .is_none_or(|reflection| reflection.epoch != epoch)
        {
          self.reflection = Some(WaterReflection::new(device, targets, epoch));
        }
      }
      _ => self.reflection = None,
    }

    self.uniform = WaterUniform::new(
      &settings,
      (frame.intensity, frame.wind, frame.rain),
      (frame.time, self.flow),
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
  pub fn is_refracting(&self) -> bool {
    self.is_drawn && self.settings.mode == RenderWaterMode::Enhanced
  }

  pub fn get_mode(&self) -> RenderWaterMode {
    self.settings.mode
  }

  pub fn get_uniform(&self) -> &WaterUniform {
    &self.uniform
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
