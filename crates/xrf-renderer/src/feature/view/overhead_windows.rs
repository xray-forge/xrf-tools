use glam::Vec4;

use crate::feature::effects::rain::cover::rain_cover_shape::RainCoverShape;

/// Where the overhead maps stand: the rain cover's window and shape, the level's surface's window and shape, and its
/// water's window.
#[derive(Clone, Copy, Debug)]
pub struct OverheadWindows {
  pub cover: Vec4,
  pub cover_shape: RainCoverShape,
  pub surface: Vec4,
  pub surface_shape: Vec4,
  pub water: Vec4,
}
