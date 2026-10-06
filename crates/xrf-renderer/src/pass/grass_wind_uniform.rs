use glam::Vec4;

use crate::pass::foliage_wind_values::FoliageWindValues;

/// How the grass sways this frame, as `shaders/grass/grass.wgsl` reads it, in the engine's space: each wave's lean across
/// the ground, then each wave's direction with its phase in `w`, both over a turn.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GrassWindUniform {
  pub wind_1: Vec4,
  pub wind_2: Vec4,
  pub wave_1: Vec4,
  pub wave_2: Vec4,
  /// The same four the frame before, which a tuft's motion is measured from.
  pub previous_wind_1: Vec4,
  pub previous_wind_2: Vec4,
  pub previous_wave_1: Vec4,
  pub previous_wave_2: Vec4,
  /// The enhanced foliage motion's wind, grass and trees strengths, and its flow fields' drift this frame and the last
  /// (`FoliageWindValues`).
  pub foliage_wind: Vec4,
  pub foliage_grass: Vec4,
  pub foliage_trees: Vec4,
  pub foliage_anim: Vec4,
  pub foliage_previous_anim: Vec4,
  pub foliage_flora: Vec4,
}

impl GrassWindUniform {
  /// This sway after the one before it, or after itself for a first frame.
  pub fn following(mut self, before: Option<&Self>) -> Self {
    let before: Self = before.copied().unwrap_or(self);

    self.previous_wind_1 = before.wind_1;
    self.previous_wind_2 = before.wind_2;
    self.previous_wave_1 = before.wave_1;
    self.previous_wave_2 = before.wave_2;
    self
  }

  /// This sway with the enhanced foliage motion's values.
  pub fn with_foliage(mut self, foliage: &FoliageWindValues) -> Self {
    self.foliage_wind = foliage.wind;
    self.foliage_grass = foliage.grass;
    self.foliage_trees = foliage.trees;
    self.foliage_anim = foliage.anim;
    self.foliage_previous_anim = foliage.previous_anim;
    self.foliage_flora = foliage.flora;
    self
  }
}
