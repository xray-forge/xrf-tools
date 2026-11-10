use xrf_level::BuildPrjPortal;
use xrf_math::Vector3d;

/// A portal's polygon as the Build simplifies one of more corners than a compiled portal holds (`CPortal::Simplify`,
/// `portal.cpp:355`): the plane fitted to its corners, the corners seen along the plane's normal, their convex hull
/// without collinear corners, the edge whose collapse adds the least area collapsed until six corners are left, and
/// those put back on the plane, wound as the portal's own corners are. Done in double precision, as the editor's
/// geometry library does most of it.
pub(crate) struct LevelBuildPortal;

/// A point on the fitted plane, in the plane's own axes.
type Point = [f64; 2];

/// The plane a portal is seen on: a point of it, and its axes, right, up and the normal.
struct PortalPlane {
  origin: [f64; 3],
  right: [f64; 3],
  up: [f64; 3],
}

impl LevelBuildPortal {
  /// The simplified polygon of `vertices`; none where the corners hold no plane or fewer than three remain.
  pub(crate) fn simplify(vertices: &[Vector3d]) -> Option<Vec<Vector3d>> {
    let points: Vec<[f64; 3]> = vertices
      .iter()
      .map(|vertex| [f64::from(vertex.x), f64::from(vertex.y), f64::from(vertex.z)])
      .collect();
    let plane: PortalPlane = Self::fit_plane(&points)?;
    let flat: Vec<Point> = points
      .iter()
      .map(|point| {
        let offset: [f64; 3] = sub(point, &plane.origin);

        [dot(&offset, &plane.right), dot(&offset, &plane.up)]
      })
      .collect();
    let mut hull: Vec<Point> = Self::get_hull(&flat);

    while hull.len() > BuildPrjPortal::MAXIMUM_VERTICES {
      if !Self::collapse_smallest_edge(&mut hull) {
        return None;
      }
    }

    if hull.len() <= 2 {
      return None;
    }

    #[expect(
      clippy::cast_possible_truncation,
      reason = "the build stores portals in single precision"
    )]
    let mut simplified: Vec<Vector3d> = hull
      .iter()
      .map(|[x, y]| {
        Vector3d::new(
          (plane.origin[0] + x * plane.right[0] + y * plane.up[0]) as f32,
          (plane.origin[1] + x * plane.right[1] + y * plane.up[1]) as f32,
          (plane.origin[2] + x * plane.right[2] + y * plane.up[2]) as f32,
        )
      })
      .collect();

    if dot(&Self::get_normal(&simplified), &Self::get_normal(vertices)) < 0.0 {
      simplified.reverse();
    }

    Some(simplified)
  }

  /// `Mgc::OrthogonalPlaneFit`: the plane through the points' mean whose normal is the direction they spread least
  /// along, and on it the axes `CPortal::Simplify` looks along (`build_camera` from the mean towards `-normal`).
  fn fit_plane(points: &[[f64; 3]]) -> Option<PortalPlane> {
    #[expect(clippy::cast_precision_loss, reason = "a portal holds a few dozen corners")]
    let count: f64 = points.len() as f64;
    let origin: [f64; 3] = points
      .iter()
      .fold([0.0; 3], |sum, point| add(&sum, point))
      .map(|sum| sum / count);
    let mut covariance: [[f64; 3]; 3] = [[0.0; 3]; 3];

    for point in points {
      let offset: [f64; 3] = sub(point, &origin);

      for (row, value) in covariance.iter_mut().zip(offset) {
        for (cell, other) in row.iter_mut().zip(offset) {
          *cell += value * other;
        }
      }
    }

    let normal: [f64; 3] = Self::get_least_eigenvector(covariance)?;
    let up_hint: [f64; 3] = if normal[1].abs() > 0.99 {
      [1.0, 0.0, 0.0]
    } else {
      [0.0, 1.0, 0.0]
    };
    // `build_camera(from, from - normal, up)`: looking along -normal, right is up × look and up look × right.
    let look: [f64; 3] = normal.map(|value| -value);
    let up: [f64; 3] = normalize(&cross(&normal, &normalize(&cross(&up_hint, &normal))?))?;
    let right: [f64; 3] = normalize(&cross(&up, &look))?;
    let up: [f64; 3] = normalize(&cross(&look, &right))?;

    Some(PortalPlane { origin, right, up })
  }

  /// The eigenvector of the least eigenvalue of a symmetric matrix, by cyclic Jacobi rotations.
  fn get_least_eigenvector(mut matrix: [[f64; 3]; 3]) -> Option<[f64; 3]> {
    let mut vectors: [[f64; 3]; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    for _ in 0..Self::JACOBI_SWEEPS {
      for (p, q) in [(0, 1), (0, 2), (1, 2)] {
        if matrix[p][q].abs() < f64::EPSILON * (matrix[p][p].abs() + matrix[q][q].abs()) {
          continue;
        }

        let theta: f64 = (matrix[q][q] - matrix[p][p]) / (2.0 * matrix[p][q]);
        let tangent: f64 = theta.signum() / (theta.abs() + theta.mul_add(theta, 1.0).sqrt());
        let cosine: f64 = 1.0 / tangent.mul_add(tangent, 1.0).sqrt();
        let sine: f64 = tangent * cosine;

        for row in &mut matrix {
          let (a, b) = (row[p], row[q]);

          row[p] = cosine * a - sine * b;
          row[q] = sine * a + cosine * b;
        }

        let (row_p, row_q) = (matrix[p], matrix[q]);

        matrix[p] = [0, 1, 2].map(|column| cosine * row_p[column] - sine * row_q[column]);
        matrix[q] = [0, 1, 2].map(|column| sine * row_p[column] + cosine * row_q[column]);

        for vector in &mut vectors {
          let (a, b) = (vector[p], vector[q]);

          vector[p] = cosine * a - sine * b;
          vector[q] = sine * a + cosine * b;
        }
      }
    }

    let least: usize = (0..3).min_by(|a, b| matrix[*a][*a].total_cmp(&matrix[*b][*b]))?;

    normalize(&[vectors[0][least], vectors[1][least], vectors[2][least]])
  }

  /// Rotation sweeps enough for a 3x3 symmetric matrix to settle in double precision.
  const JACOBI_SWEEPS: usize = 32;

  /// The convex hull of `points` counterclockwise, no collinear point kept (`ConvexHull2D::ByIncremental` and
  /// `RemoveCollinear`), by Andrew's monotone chain.
  fn get_hull(points: &[Point]) -> Vec<Point> {
    let mut sorted: Vec<Point> = points.to_vec();

    sorted.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    sorted.dedup();

    if sorted.len() < 3 {
      return sorted;
    }

    let turn = |o: &Point, a: &Point, b: &Point| (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]);
    let mut hull: Vec<Point> = Vec::with_capacity(sorted.len() * 2);

    for pass in [&sorted[..], &sorted.iter().rev().copied().collect::<Vec<_>>()[..]] {
      let start: usize = hull.len();

      for point in pass {
        while hull.len() >= start + 2 && turn(&hull[hull.len() - 2], &hull[hull.len() - 1], point) <= 0.0 {
          hull.pop();
        }

        hull.push(*point);
      }

      hull.pop();
    }

    hull
  }

  /// `SimplifyVertices` (`portal.cpp:307`): of each edge, the point its two neighbouring edges meet at and the area of
  /// the triangle it adds; the edge of the least area collapses into its point. False where no neighbouring edges
  /// meet.
  fn collapse_smallest_edge(vertices: &mut Vec<Point>) -> bool {
    let count: usize = vertices.len();
    let mut smallest: Option<(usize, f64, Point)> = None;

    for index in 0..count {
      let before: Point = vertices[(index + count - 1) % count];
      let (first, second) = (vertices[index], vertices[(index + 1) % count]);
      let after: Point = vertices[(index + 2) % count];
      let Some(meet) = Self::intersect(&before, &first, &after, &second) else {
        continue;
      };
      let area: f64 = Self::get_area(&first, &meet, &second);

      if smallest.is_none_or(|(_, least, _)| area < least) {
        smallest = Some((index, area, meet));
      }
    }

    let Some((index, _, meet)) = smallest else {
      return false;
    };

    vertices[index] = meet;
    vertices.remove((index + 1) % count);

    true
  }

  /// `Isect2DLvs2DL`: where the line through `a` and `b` meets the one through `c` and `d`; none for lines nearly
  /// parallel.
  fn intersect(a: &Point, b: &Point, c: &Point, d: &Point) -> Option<Point> {
    let (a1, b1, c1) = (b[1] - a[1], a[0] - b[0], b[0] * a[1] - a[0] * b[1]);
    let (a2, b2, c2) = (d[1] - c[1], c[0] - d[0], d[0] * c[1] - c[0] * d[1]);
    let denominator: f64 = a1 * b2 - a2 * b1;

    if denominator.abs() < f64::from(xrf_math::EPS) {
      return None;
    }

    Some([(b1 * c2 - b2 * c1) / denominator, (a2 * c1 - a1 * c2) / denominator])
  }

  /// `tri_area`: Heron's formula.
  fn get_area(a: &Point, b: &Point, c: &Point) -> f64 {
    let distance = |p: &Point, q: &Point| (p[0] - q[0]).hypot(p[1] - q[1]);
    let (first, second, third) = (distance(a, b), distance(b, c), distance(c, a));
    let half: f64 = (first + second + third) / 2.0;

    (half * (half - first) * (half - second) * (half - third)).sqrt()
  }

  /// The way a polygon winds, as `CPortal::Update` takes it: the sum of each consecutive pair's turn about the middle
  /// (`mknormal`), the last corner not joined back to the first.
  fn get_normal(vertices: &[Vector3d]) -> [f64; 3] {
    let points: Vec<[f64; 3]> = vertices
      .iter()
      .map(|vertex| [f64::from(vertex.x), f64::from(vertex.y), f64::from(vertex.z)])
      .collect();
    #[expect(clippy::cast_precision_loss, reason = "a portal holds a few dozen corners")]
    let centre: [f64; 3] = points
      .iter()
      .fold([0.0; 3], |sum, point| add(&sum, point))
      .map(|sum| sum / points.len() as f64);

    points.windows(2).fold([0.0; 3], |sum, pair| {
      normalize(&cross(&sub(&pair[0], &centre), &sub(&pair[1], &centre))).map_or(sum, |turn| add(&sum, &turn))
    })
  }
}

fn add(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
  [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
  [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
  a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
  [
    a[1] * b[2] - a[2] * b[1],
    a[2] * b[0] - a[0] * b[2],
    a[0] * b[1] - a[1] * b[0],
  ]
}

fn normalize(a: &[f64; 3]) -> Option<[f64; 3]> {
  let length: f64 = dot(a, a).sqrt();

  (length > f64::EPSILON).then(|| a.map(|value| value / length))
}
