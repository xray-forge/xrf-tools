use serde::Serialize;
use xrf_math::Vector3d;

use crate::data::visual::skeleton::bind_transform::BindTransform;

/// One transform in renderer space: three basis vectors and a translation.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualTransform {
  pub i: Vector3d,
  pub j: Vector3d,
  pub k: Vector3d,
  pub c: Vector3d,
}

impl VisualTransform {
  /// Floats one transform takes: its basis, then its translation.
  pub const FLOATS: usize = 12;

  /// Where a spawned object stands, `CGameObject::net_Spawn`'s `XFORM`: `setXYZ(o_Angle)` at `o_Position`.
  pub fn of_spawn(position: &Vector3d, angle: &Vector3d) -> Self {
    BindTransform::from_angle(angle, position).to_renderer_space()
  }

  /// The same transform mirrored through `z`, which takes it between renderer and engine space either way.
  pub fn mirrored(&self) -> Self {
    BindTransform::from_renderer_space(self).mirrored().to_visual()
  }

  /// Its basis and translation as a matrix's sixteen floats, column by column.
  pub fn to_matrix(&self) -> [f32; 16] {
    let Self { i, j, k, c } = self;

    [
      i.x, i.y, i.z, 0.0, j.x, j.y, j.z, 0.0, k.x, k.y, k.z, 0.0, c.x, c.y, c.z, 1.0,
    ]
  }

  /// A transform out of its twelve floats, basis then translation, as a baked motion's frame holds one.
  pub fn from_floats(floats: &[f32; Self::FLOATS]) -> Self {
    Self {
      i: Vector3d::new(floats[0], floats[1], floats[2]),
      j: Vector3d::new(floats[3], floats[4], floats[5]),
      k: Vector3d::new(floats[6], floats[7], floats[8]),
      c: Vector3d::new(floats[9], floats[10], floats[11]),
    }
  }

  /// Its twelve floats, basis then translation.
  pub fn to_floats(&self) -> [f32; Self::FLOATS] {
    let Self { i, j, k, c } = self;

    [i.x, i.y, i.z, j.x, j.y, j.z, k.x, k.y, k.z, c.x, c.y, c.z]
  }

  /// A point of its own space where the transform stands it, `transform_tiny`.
  pub fn apply_to_point(&self, point: &Vector3d) -> Vector3d {
    let Self { i, j, k, c } = self;

    Vector3d::new(
      c.x + i.x * point.x + j.x * point.y + k.x * point.z,
      c.y + i.y * point.x + j.y * point.y + k.y * point.z,
      c.z + i.z * point.x + j.z * point.y + k.z * point.z,
    )
  }
}
