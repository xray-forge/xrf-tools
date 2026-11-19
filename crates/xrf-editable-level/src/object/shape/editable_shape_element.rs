use xrf_math::{Affine3d, Vector3d};

/// One element of a shape, `CShapeData::shape_def`, in the object's own space.
#[derive(Clone, Debug, PartialEq)]
pub enum EditableShapeElement {
  Sphere {
    center: Vector3d,
    radius: f32,
  },
  /// The matrix the unit box (`-0.5` to `0.5` on each axis) is placed by.
  Box(Affine3d),
}
