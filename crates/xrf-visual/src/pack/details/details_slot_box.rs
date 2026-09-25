/// An edge of a triangle, and the pairs of its corners its three axis tests project.
type EdgeAxes = ([f32; 3], [[f32; 3]; 2], [[f32; 3]; 2], [[f32; 3]; 2]);

/// A detail slot's box as `CDetailManager::cache_Task` makes it, grown by `EPS_L`, and the test the engine's box
/// query keeps a collision triangle by (`xrCDB/xrCDB_box.cpp`, `box_collider::_tri` with class III tests).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DetailsSlotBox {
  center: [f32; 3],
  extents: [f32; 3],
}

impl DetailsSlotBox {
  /// Metres a slot spans across, `dm_slot_size`.
  pub const SLOT_METERS: f32 = 2.0;

  /// `EPS_L`, which the box grows by on every side.
  const GROWTH: f32 = 0.001;

  /// The box of world slot `(x, z)`, from its base height to its top.
  pub fn of(x: i32, z: i32, base: f32, height: f32) -> Self {
    let minimum: [f32; 3] = [
      x as f32 * Self::SLOT_METERS - Self::GROWTH,
      base - Self::GROWTH,
      z as f32 * Self::SLOT_METERS - Self::GROWTH,
    ];
    let maximum: [f32; 3] = [
      x as f32 * Self::SLOT_METERS + Self::SLOT_METERS + Self::GROWTH,
      base + height + Self::GROWTH,
      z as f32 * Self::SLOT_METERS + Self::SLOT_METERS + Self::GROWTH,
    ];

    Self {
      center: [0, 1, 2].map(|axis| (minimum[axis] + maximum[axis]) * 0.5),
      extents: [0, 1, 2].map(|axis| (maximum[axis] - minimum[axis]) * 0.5),
    }
  }

  /// Whether a triangle overlaps the box, by the separating axis tests the engine's full box query runs.
  pub fn overlaps(&self, triangle: &[[f32; 3]; 3]) -> bool {
    let [e_x, e_y, e_z] = self.extents;
    let [v0, v1, v2] = triangle.map(|corner| [0, 1, 2].map(|axis| corner[axis] - self.center[axis]));

    for axis in 0..3 {
      let minimum: f32 = v0[axis].min(v1[axis]).min(v2[axis]);
      let maximum: f32 = v0[axis].max(v1[axis]).max(v2[axis]);

      if minimum > self.extents[axis] || maximum < -self.extents[axis] {
        return false;
      }
    }

    let e0: [f32; 3] = subtract(v1, v0);
    let e1: [f32; 3] = subtract(v2, v1);
    let normal: [f32; 3] = cross(e0, e1);

    if !plane_box_overlap(normal, -dot(normal, v0), self.extents) {
      return false;
    }

    // `mLeafVerts[0] - mLeafVerts[2]`, from the unmoved corners, which is the same edge.
    let e2: [f32; 3] = subtract(v0, v2);
    // The corners each edge's three axes project, as `AXISTEST_X01`, `_X2`, `_Y02`, `_Y1`, `_Z12` and `_Z0` pick them.
    let axes: [EdgeAxes; 3] = [
      (e0, [v0, v2], [v0, v2], [v1, v2]),
      (e1, [v0, v2], [v0, v2], [v0, v1]),
      (e2, [v0, v1], [v0, v1], [v1, v2]),
    ];

    for (edge, on_x, on_y, on_z) in axes {
      let [f_x, f_y, f_z] = edge.map(f32::abs);
      let project_x = |v: [f32; 3]| edge[2] * v[1] - edge[1] * v[2];
      let project_y = |v: [f32; 3]| edge[0] * v[2] - edge[2] * v[0];
      let project_z = |v: [f32; 3]| edge[1] * v[0] - edge[0] * v[1];

      if is_separated(project_x(on_x[0]), project_x(on_x[1]), f_z * e_y + f_y * e_z)
        || is_separated(project_y(on_y[0]), project_y(on_y[1]), f_z * e_x + f_x * e_z)
        || is_separated(project_z(on_z[0]), project_z(on_z[1]), f_y * e_x + f_x * e_y)
      {
        return false;
      }
    }

    true
  }
}

/// Whether two projections of a triangle both miss the box's projection of that radius.
fn is_separated(first: f32, second: f32, radius: f32) -> bool {
  first.min(second) > radius || first.max(second) < -radius
}

/// `planeBoxOverlap`: whether the plane `normal . x + d = 0` passes through the box about the origin.
fn plane_box_overlap(normal: [f32; 3], d: f32, extents: [f32; 3]) -> bool {
  let mut minimum: [f32; 3] = [0.0; 3];
  let mut maximum: [f32; 3] = [0.0; 3];

  for axis in 0..3 {
    if normal[axis] > 0.0 {
      minimum[axis] = -extents[axis];
      maximum[axis] = extents[axis];
    } else {
      minimum[axis] = extents[axis];
      maximum[axis] = -extents[axis];
    }
  }

  if dot(normal, minimum) + d > 0.0 {
    return false;
  }

  dot(normal, maximum) + d >= 0.0
}

fn subtract(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
  [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
  [
    a[1] * b[2] - a[2] * b[1],
    a[2] * b[0] - a[0] * b[2],
    a[0] * b[1] - a[1] * b[0],
  ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
  a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
