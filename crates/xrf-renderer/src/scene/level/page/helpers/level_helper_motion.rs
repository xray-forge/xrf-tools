use xrf_math::glam::{Mat4, Vec3};

use crate::host::level::helpers::render_helper_shape::RenderHelperShape;

/// A helped object an object motion carries, kept to stand it again where the motion has its object: its place among
/// the scene's helped objects, the motion, its axes and shapes in its own space, and where it stands now.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LevelHelperMotion {
  /// By index among the scene's helped objects.
  pub object: usize,
  pub motion: String,
  /// Its transform with no translation, which the motion's place is put under.
  pub axes: Mat4,
  pub shapes: Vec<RenderHelperShape>,
  /// Renderer space.
  pub place: Vec3,
}
