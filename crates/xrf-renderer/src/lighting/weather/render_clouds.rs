use glam::Vec4;

/// The clouds, as the two keyframes either side of the time name them (`s_clouds0`, `s_clouds1`).
#[derive(Clone, Debug, PartialEq)]
pub struct RenderClouds {
  /// The two keyframes' `clouds_texture`, none for one naming none, which draws nothing.
  pub textures: [Option<String>; 2],
  /// `clouds_color` as the engine holds it: the colour scaled by its multiplier, then the cover in alpha.
  pub color: Vec4,
  /// `clouds_rotation`, in radians about the vertical.
  pub rotation: f32,
}
