use xrf_math::Matrix4x4;

use crate::light::vertex::light_tree_color::LightTreeColor;

/// One placement of a multi-use model's visual, an `MT_TREE` visual (`OGF_Reference`): drawn by the reference's
/// transform, lit through its scale and bias.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryTree {
  /// The model, by index into `build.prj`'s multi-use models.
  pub model: u32,
  /// The model's visual it places.
  pub visual: u32,
  /// The reference, by index into `build.prj`'s multi-use references.
  pub reference: u32,
  pub sector: u16,
  pub transform: Matrix4x4,
  /// `c_scale`.
  pub scale: LightTreeColor,
  /// `c_bias`.
  pub bias: LightTreeColor,
}
