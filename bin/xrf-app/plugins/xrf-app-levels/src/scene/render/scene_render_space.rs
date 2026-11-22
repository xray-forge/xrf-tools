use xrf_editable_level::EditableObjectTransform;
use xrf_math::glam::{Mat4, Vec3};
use xrf_math::{Affine3d, Vector3d};
use xrf_visual::{VisualTransform, convert_vector};

/// The mapping from a scene's saved engine space into renderer space: mirrored through `z`, as a model's vertices are,
/// so whatever the scene places is drawn where its models stand.
pub(crate) struct SceneRenderSpace;

impl SceneRenderSpace {
  /// Places a thing given in its own renderer space where `transform` puts it, as sixteen column-major floats: the
  /// world transform `CCustomObject::OnUpdateTransform` composes and the level build places mesh points by, mirrored
  /// through `z` on both sides.
  pub(crate) fn place(transform: &EditableObjectTransform) -> [f32; 16] {
    let placement: Affine3d = transform.to_matrix().to_affine();

    VisualTransform {
      i: placement.x_axis,
      j: placement.y_axis,
      k: placement.z_axis,
      c: placement.translation,
    }
    .mirrored()
    .to_matrix()
  }

  /// Places a thing at the engine space point `position`, unrotated and unscaled.
  pub(crate) fn place_at(position: &Vector3d) -> [f32; 16] {
    Mat4::from_translation(Self::convert_point(position)).to_cols_array()
  }

  /// A point or vector of engine space in renderer space.
  pub(crate) fn convert_point(point: &Vector3d) -> Vec3 {
    Vec3::from(convert_vector(point))
  }
}
