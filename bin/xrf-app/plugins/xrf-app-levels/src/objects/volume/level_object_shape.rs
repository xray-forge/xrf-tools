use serde::Serialize;
use xrf_math::Vector3d;
use xrf_spawn::Shape;

use crate::objects::LevelObject;

/// One shape of a logic volume in its object's own space, renderer space: what `CCF_Shape` tests against.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum LevelObjectShape {
  Sphere {
    center: Vector3d,
    radius: f32,
  },
  /// The unit cube about the origin taken through `axes` and moved to `center`.
  Box {
    axes: [Vector3d; 3],
    center: Vector3d,
  },
}

impl From<&Shape> for LevelObjectShape {
  /// The spawn's shape, engine space, mirrored as the object's transform is: the unit cube is symmetric, so a box keeps
  /// its corners with each axis mirrored.
  fn from(shape: &Shape) -> Self {
    match shape {
      Shape::Sphere(sphere) => Self::Sphere {
        center: LevelObject::to_renderer_space(&sphere.center),
        radius: sphere.radius,
      },
      Shape::Box(transform) => Self::Box {
        axes: [&transform.x_axis, &transform.y_axis, &transform.z_axis].map(LevelObject::to_renderer_space),
        center: LevelObject::to_renderer_space(&transform.translation),
      },
    }
  }
}
