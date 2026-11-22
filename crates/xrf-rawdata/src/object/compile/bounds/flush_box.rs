use xrf_geometry::{ConvexOutline, SymmetricEigen};
use xrf_math::glam::{DVec2, DVec3};
use xrf_math::{Obb, Vector3d};

use crate::object::compile::bounds::convex_hull::ConvexHull;

/// The least-volume box around a point set with a face flush to a face of its convex hull, or along its principal axes
/// where that is smaller or the points span no volume: one answer for one input, with no search.
///
/// It stands in for the export's Wild Magic `MinBox3`, whose Powell search moves to another local minimum for a
/// last-bit change of its input: on vanilla's bones this one is 11% to 15% smaller and never leaves a point out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FlushBox {
  /// Three orthonormal axes, right-handed.
  pub(crate) axes: [DVec3; 3],
  pub(crate) center: DVec3,
  /// Half its extent along each axis.
  pub(crate) half_extents: DVec3,
}

impl FlushBox {
  /// The thickness, against the points' extent, every box is compared as having at least.
  const RELATIVE_THICKNESS: f64 = 1e-9;

  /// The box around `points`, or nothing when there are none. Its extents are taken over every point, since the hull
  /// leaves out points within its tolerance of a face.
  pub(crate) fn around(points: &[DVec3]) -> Option<Self> {
    if points.is_empty() {
      return None;
    }

    let low: DVec3 = points.iter().fold(DVec3::MAX, |low, point| low.min(*point));
    let high: DVec3 = points.iter().fold(DVec3::MIN, |high, point| high.max(*point));
    let floor: f64 = Self::RELATIVE_THICKNESS * (high - low).max_element();
    let principal: [DVec3; 3] = Self::principal_axes(points);
    let mut best: Self = Self::along(points, principal);

    for axis in principal {
      best = Self::smaller(best, Self::flush_to(points, axis, floor), floor);
    }

    if let Some(hull) = ConvexHull::of(points) {
      let corners: Vec<DVec3> = hull.vertices().iter().map(|index| points[*index]).collect();

      for [a, b, c] in &hull.faces {
        let normal: DVec3 = (points[*b] - points[*a]).cross(points[*c] - points[*a]);

        best = Self::smaller(best, Self::flush_to(&corners, normal, floor), floor);
      }
    }

    Some(Self::along(points, best.axes))
  }

  #[cfg(test)]
  pub(crate) fn volume(&self) -> f64 {
    8.0 * self.half_extents.element_product()
  }

  /// The volume boxes are compared by, each half extent at least `floor`: flat points give every box no volume, and
  /// the floor compares those by area instead, leaving a box of any real thickness as it is.
  fn size(&self, floor: f64) -> f64 {
    self.half_extents.max(DVec3::splat(floor)).element_product()
  }

  fn smaller(best: Self, candidate: Option<Self>, floor: f64) -> Self {
    candidate.filter(|it| it.size(floor) < best.size(floor)).unwrap_or(best)
  }

  /// The box along `axes` holding every point.
  fn along(points: &[DVec3], axes: [DVec3; 3]) -> Self {
    let project = |point: &DVec3| DVec3::new(point.dot(axes[0]), point.dot(axes[1]), point.dot(axes[2]));
    let (low, high) = points
      .iter()
      .map(project)
      .fold((DVec3::MAX, DVec3::MIN), |(low, high), projected| {
        (low.min(projected), high.max(projected))
      });
    let middle: DVec3 = (low + high) * 0.5;

    Self {
      axes,
      center: axes[0] * middle.x + axes[1] * middle.y + axes[2] * middle.z,
      half_extents: (high - low) * 0.5,
    }
  }

  /// The smallest box with one axis along `normal`: the smallest rectangle around the points projected on the plane
  /// across it, with a side along each edge of their 2D hull in turn.
  ///
  /// A rectangle's extremes lie on the 2D hull and its depth along `normal` is the same for every edge, so each edge is
  /// measured over the outline alone, and only the chosen one over every point.
  fn flush_to(points: &[DVec3], normal: DVec3, floor: f64) -> Option<Self> {
    let normal: DVec3 = normal.try_normalize()?;
    let helper: DVec3 = if normal.x.abs() < 0.9 { DVec3::X } else { DVec3::Y };
    let u: DVec3 = normal.cross(helper).try_normalize()?;
    let v: DVec3 = normal.cross(u);
    let planar: Vec<DVec2> = points
      .iter()
      .map(|point| DVec2::new(point.dot(u), point.dot(v)))
      .collect();
    let depth: f64 = Self::half_span(points.iter().map(|point| point.dot(normal))).max(floor);
    let outline: Vec<DVec2> = ConvexOutline::of(&planar);
    let mut best: Option<(f64, DVec2)> = None;

    for (index, a) in outline.iter().enumerate() {
      let Some(edge) = (outline[(index + 1) % outline.len()] - *a).try_normalize() else {
        continue;
      };
      // `normal × along` is the edge turned a quarter in the plane, so the side runs along `edge.perp()`.
      let size: f64 = [edge, edge.perp()]
        .map(|axis| Self::half_span(outline.iter().map(|point| point.dot(axis))).max(floor))
        .iter()
        .product::<f64>()
        * depth;

      if best.is_none_or(|(best, _)| size < best) {
        best = Some((size, edge));
      }
    }

    let (_, edge) = best?;
    let along: DVec3 = u * edge.x + v * edge.y;

    Some(Self::along(points, [along, normal.cross(along), normal]))
  }

  /// Half the distance between the least and the greatest of `values`.
  fn half_span(values: impl Iterator<Item = f64>) -> f64 {
    let (low, high) = values.fold((f64::MAX, f64::MIN), |(low, high), value| {
      (low.min(value), high.max(value))
    });

    0.5 * (high - low)
  }

  /// The principal axes of the points, the eigenvectors of their covariance, right-handed.
  fn principal_axes(points: &[DVec3]) -> [DVec3; 3] {
    let count: f64 = points.len() as f64;
    let mean: DVec3 = points.iter().sum::<DVec3>() / count;
    let mut matrix: [[f64; 3]; 3] = [[0.0; 3]; 3];

    for point in points {
      let offset: DVec3 = *point - mean;

      for (row, values) in matrix.iter_mut().enumerate() {
        for (column, value) in values.iter_mut().enumerate() {
          *value += offset[row] * offset[column] / count;
        }
      }
    }

    let [first, second, _] = SymmetricEigen::of(matrix).vectors;

    [first, second, first.cross(second)]
  }
}

/// The engine's `Fobb`: its rows are the box's axes.
impl From<&FlushBox> for Obb {
  fn from(fitted: &FlushBox) -> Self {
    let single = |vector: DVec3| Vector3d::from(vector.as_vec3());

    Self {
      rotation: fitted.axes.map(single),
      center: single(fitted.center),
      half_extents: single(fitted.half_extents),
    }
  }
}
