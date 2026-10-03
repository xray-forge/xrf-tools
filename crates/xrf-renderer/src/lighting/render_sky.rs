use glam::Vec3;

use crate::lighting::render_clouds::RenderClouds;

/// The sky, as the two keyframes either side of the time name it (`$user$sky0`, `$user$sky1`).
#[derive(Clone, Debug, PartialEq)]
pub struct RenderSky {
  /// The two keyframes' `sky_texture` cubes, none for one naming none.
  pub textures: [Option<String>; 2],
  /// Their `#small` irradiance cubes, which light the hemisphere (`env_s0`, `env_s1`).
  pub environments: [Option<String>; 2],
  /// How far from the first to the second, `L_ambient.w`.
  pub blend: f32,
  /// `sky_color`: what the cubes are multiplied by.
  pub color: Vec3,
  /// `sky_rotation`, in radians about the vertical.
  pub rotation: f32,
  pub clouds: RenderClouds,
  /// The lens flare the nearer keyframe names by `sun`, as `CEnvDescriptorMixer::lerp` takes it; none for one naming
  /// none.
  pub sun: Option<String>,
}
