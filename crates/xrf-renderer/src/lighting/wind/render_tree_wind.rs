use glam::Vec3;

/// How the trees sway, in the terms a weather keyframe uses (`CEnvDescriptor::m_fTree*`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderTreeWind {
  /// `trees_amplitude`: how far a tree leans, a share of its height.
  pub amplitude: f32,
  /// `trees_speed`: how fast the wave runs through the level.
  pub speed: f32,
  /// `trees_rotation`: seconds the wind takes to turn once around.
  pub rotation: f32,
  /// `trees_wave`: the wave's direction through the level, in the engine's own axes.
  pub wave: Vec3,
}

impl Default for RenderTreeWind {
  /// The engine's own sway, where a weather states none (`CEnvDescriptor::load`).
  fn default() -> Self {
    Self {
      amplitude: 0.005,
      speed: 1.0,
      rotation: 10.0,
      wave: Vec3::new(0.1, 0.01, 0.11),
    }
  }
}
