use xrf_level::DETAIL_SLOT_METERS;
use xrf_math::EPS_L;
use xrf_math::glam::Vec3;

/// An edge of a triangle, and the pairs of its corners its three axis tests project.
type EdgeAxes = (Vec3, [Vec3; 2], [Vec3; 2], [Vec3; 2]);

/// A detail slot's box as `CDetailManager::cache_Task` makes it, grown by `EPS_L`, and the test the engine's box
/// query keeps a collision triangle by (`xrCDB/xrCDB_box.cpp`, `box_collider::_tri` with class III tests).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DetailsSlotBox {
  center: Vec3,
  extents: Vec3,
}

impl DetailsSlotBox {
  /// `EPS_L`, which the box grows by on every side.
  const GROWTH: f32 = EPS_L;

  /// The box of world slot `(x, z)`, from its base height to its top.
  pub(crate) fn of(x: i32, z: i32, base: f32, height: f32) -> Self {
    let minimum: [f32; 3] = [
      x as f32 * DETAIL_SLOT_METERS - Self::GROWTH,
      base - Self::GROWTH,
      z as f32 * DETAIL_SLOT_METERS - Self::GROWTH,
    ];
    let maximum: [f32; 3] = [
      x as f32 * DETAIL_SLOT_METERS + DETAIL_SLOT_METERS + Self::GROWTH,
      base + height + Self::GROWTH,
      z as f32 * DETAIL_SLOT_METERS + DETAIL_SLOT_METERS + Self::GROWTH,
    ];

    Self {
      center: Vec3::from([0, 1, 2].map(|axis| f32::midpoint(minimum[axis], maximum[axis]))),
      extents: Vec3::from([0, 1, 2].map(|axis| (maximum[axis] - minimum[axis]) * 0.5)),
    }
  }

  /// Whether a triangle overlaps the box, by the separating axis tests the engine's full box query runs.
  pub(crate) fn overlaps(&self, triangle: &[[f32; 3]; 3]) -> bool {
    let [e_x, e_y, e_z] = self.extents.to_array();
    let [v0, v1, v2] = triangle.map(|corner| Vec3::from(corner) - self.center);

    for axis in 0..3 {
      let minimum: f32 = v0[axis].min(v1[axis]).min(v2[axis]);
      let maximum: f32 = v0[axis].max(v1[axis]).max(v2[axis]);

      if minimum > self.extents[axis] || maximum < -self.extents[axis] {
        return false;
      }
    }

    let e0: Vec3 = v1 - v0;
    let e1: Vec3 = v2 - v1;
    let normal: Vec3 = e0.cross(e1);

    if !Self::plane_box_overlap(normal, -normal.dot(v0), self.extents) {
      return false;
    }

    // `mLeafVerts[0] - mLeafVerts[2]`, from the unmoved corners, which is the same edge.
    let e2: Vec3 = v0 - v2;
    // The corners each edge's three axes project, as `AXISTEST_X01`, `_X2`, `_Y02`, `_Y1`, `_Z12` and `_Z0` pick them.
    let axes: [EdgeAxes; 3] = [
      (e0, [v0, v2], [v0, v2], [v1, v2]),
      (e1, [v0, v2], [v0, v2], [v0, v1]),
      (e2, [v0, v1], [v0, v1], [v1, v2]),
    ];

    for (edge, on_x, on_y, on_z) in axes {
      let [f_x, f_y, f_z] = edge.abs().to_array();
      let project_x = |v: Vec3| edge.z * v.y - edge.y * v.z;
      let project_y = |v: Vec3| edge.x * v.z - edge.z * v.x;
      let project_z = |v: Vec3| edge.y * v.x - edge.x * v.y;

      if Self::is_separated(project_x(on_x[0]), project_x(on_x[1]), f_z * e_y + f_y * e_z)
        || Self::is_separated(project_y(on_y[0]), project_y(on_y[1]), f_z * e_x + f_x * e_z)
        || Self::is_separated(project_z(on_z[0]), project_z(on_z[1]), f_y * e_x + f_x * e_y)
      {
        return false;
      }
    }

    true
  }

  /// Whether two projections of a triangle both miss the box's projection of that radius.
  fn is_separated(first: f32, second: f32, radius: f32) -> bool {
    first.min(second) > radius || first.max(second) < -radius
  }

  /// `planeBoxOverlap`: whether the plane `normal . x + d = 0` passes through the box about the origin.
  fn plane_box_overlap(normal: Vec3, d: f32, extents: Vec3) -> bool {
    let mut minimum: Vec3 = Vec3::ZERO;
    let mut maximum: Vec3 = Vec3::ZERO;

    for axis in 0..3 {
      if normal[axis] > 0.0 {
        minimum[axis] = -extents[axis];
        maximum[axis] = extents[axis];
      } else {
        minimum[axis] = extents[axis];
        maximum[axis] = -extents[axis];
      }
    }

    if normal.dot(minimum) + d > 0.0 {
      return false;
    }

    normal.dot(maximum) + d >= 0.0
  }
}
