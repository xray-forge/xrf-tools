use serde::Serialize;
use xrf_editable_level::EditableShapeElement;
use xrf_math::Vector3d;

/// One element of a shape, in its object's own space.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub(crate) enum SceneShapeElement {
  Sphere {
    center: Vector3d,
    radius: f32,
  },
  /// The unit box, `-0.5` to `0.5` on each axis, placed by its axes and translation.
  Box {
    x_axis: Vector3d,
    y_axis: Vector3d,
    z_axis: Vector3d,
    translation: Vector3d,
  },
}

impl From<&EditableShapeElement> for SceneShapeElement {
  fn from(element: &EditableShapeElement) -> Self {
    match element {
      EditableShapeElement::Sphere { center, radius } => Self::Sphere {
        center: *center,
        radius: *radius,
      },
      EditableShapeElement::Box(matrix) => Self::Box {
        x_axis: matrix.x_axis,
        y_axis: matrix.y_axis,
        z_axis: matrix.z_axis,
        translation: matrix.translation,
      },
    }
  }
}
