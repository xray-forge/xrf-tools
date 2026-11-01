use glam::Vec4;

/// Where the overhead maps stand: the rain cover's window and its texels and reach, the level's surface's window and
/// shape, and its water's window.
#[derive(Clone, Copy, Debug)]
pub struct OverheadWindows {
  pub cover: Vec4,
  pub cover_shape: (f32, f32),
  pub surface: Vec4,
  pub surface_shape: Vec4,
  pub water: Vec4,
}
