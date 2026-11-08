use glam::{DVec2, DVec3};
use xrf_math::{Obb, Vector3d};

use crate::object::compile::bounds::convex_hull::ConvexHull;

/// A box around a point set with a face flush to a face of the points' convex hull: of the boxes so placed, the one of
/// least volume, each found as the smallest rectangle around the hull projected on that face's plane with a side along
/// an edge of the projection's own hull; the boxes along the points' principal axes stand in where they are smaller, or
/// where the points span no volume. The minimal box has a face flush to a hull face in practice, so this lands on or
/// near it, with one answer for one input and no search.
///
/// It replaces the export's Wild Magic `MinBox3`, whose Powell search lands in another local minimum for a last-bit
/// change of its input: on vanilla's bones this one is 11% to 15% smaller in volume and never leaves a point out.
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
    let mut planar: Vec<DVec2> = points
      .iter()
      .map(|point| DVec2::new(point.dot(u), point.dot(v)))
      .collect();
    let depth: f64 = Self::half_span(points.iter().map(|point| point.dot(normal))).max(floor);
    let outline: Vec<DVec2> = Self::hull_2d(&mut planar);
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

  /// The convex hull of 2D points, counter-clockwise, by Andrew's monotone chain.
  fn hull_2d(points: &mut [DVec2]) -> Vec<DVec2> {
    points.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));

    let turn = |o: DVec2, a: DVec2, b: DVec2| (a - o).perp_dot(b - o);
    let mut outline: Vec<DVec2> = Vec::with_capacity(points.len() + 1);

    // The lower chain left to right, then the upper chain back, never popping into the lower one.
    for point in points.iter() {
      while outline.len() >= 2 && turn(outline[outline.len() - 2], outline[outline.len() - 1], *point) <= 0.0 {
        outline.pop();
      }

      outline.push(*point);
    }

    let lower: usize = outline.len() + 1;

    for point in points.iter().rev().skip(1) {
      while outline.len() >= lower && turn(outline[outline.len() - 2], outline[outline.len() - 1], *point) <= 0.0 {
        outline.pop();
      }

      outline.push(*point);
    }

    outline.pop();
    outline
  }

  /// The principal axes of the points, the eigenvectors of their covariance by Jacobi rotations, right-handed.
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

    let mut vectors: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    for _ in 0..32 {
      for (p, q) in [(0, 1), (0, 2), (1, 2)] {
        if matrix[p][q].abs() < 1e-15 {
          continue;
        }

        let theta: f64 = 0.5 * (matrix[q][q] - matrix[p][p]) / matrix[p][q];
        let t: f64 = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
        let (c, s) = (1.0 / (t * t + 1.0).sqrt(), t / (t * t + 1.0).sqrt());

        for row in matrix.iter_mut().chain(vectors.iter_mut()) {
          let (kp, kq) = (row[p], row[q]);

          row[p] = c * kp - s * kq;
          row[q] = s * kp + c * kq;
        }

        // `p` is always below `q`, so the two rows split apart.
        let (low, high) = matrix.split_at_mut(q);

        for (pk, qk) in low[p].iter_mut().zip(high[0].iter_mut()) {
          let (a, b) = (*pk, *qk);

          *pk = c * a - s * b;
          *qk = s * a + c * b;
        }
      }
    }

    let column = |index: usize| DVec3::new(vectors[0][index], vectors[1][index], vectors[2][index]);
    let (first, second) = (column(0), column(1));

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
