use glam::{DVec2, DVec3, Vec3};
use xrf_geometry::{ConvexOutline, SymmetricEigen};
use xrf_level::BuildPrjPortal;
use xrf_math::Vector3d;

/// A portal's polygon as the Build simplifies one of more corners than a compiled portal holds (`CPortal::Simplify`,
/// `portal.cpp:355`): the plane fitted to its corners, the corners seen along the plane's normal, their convex hull
/// without collinear corners, the edge whose collapse adds the least area collapsed until six corners are left, and
/// those put back on the plane, wound as the portal's own corners are. Done in double precision, as the editor's
/// geometry library does most of it.
pub(crate) struct LevelBuildPortal;

/// The plane a portal is seen on: a point of it, and its axes, right, up and the normal.
struct PortalPlane {
  origin: DVec3,
  right: DVec3,
  up: DVec3,
}

impl LevelBuildPortal {
  /// The simplified polygon of `vertices`; none where the corners hold no plane or fewer than three remain.
  pub(crate) fn simplify(vertices: &[Vector3d]) -> Option<Vec<Vector3d>> {
    let points: Vec<DVec3> = vertices.iter().map(|vertex| Vec3::from(vertex).as_dvec3()).collect();
    let plane: PortalPlane = Self::fit_plane(&points)?;
    let flat: Vec<DVec2> = points
      .iter()
      .map(|point| {
        let offset: DVec3 = *point - plane.origin;

        DVec2::new(offset.dot(plane.right), offset.dot(plane.up))
      })
      .collect();
    let mut hull: Vec<DVec2> = ConvexOutline::of(&flat);

    while hull.len() > BuildPrjPortal::MAXIMUM_VERTICES {
      if !Self::collapse_smallest_edge(&mut hull) {
        return None;
      }
    }

    if hull.len() <= 2 {
      return None;
    }

    let mut simplified: Vec<Vector3d> = hull
      .iter()
      .map(|point| Vector3d::from((plane.origin + point.x * plane.right + point.y * plane.up).as_vec3()))
      .collect();

    if Self::get_normal(&simplified).dot(Self::get_normal(vertices)) < 0.0 {
      simplified.reverse();
    }

    Some(simplified)
  }

  /// `Mgc::OrthogonalPlaneFit`: the plane through the points' mean whose normal is the direction they spread least
  /// along, and on it the axes `CPortal::Simplify` looks along (`build_camera` from the mean towards `-normal`).
  fn fit_plane(points: &[DVec3]) -> Option<PortalPlane> {
    #[expect(clippy::cast_precision_loss, reason = "a portal holds a few dozen corners")]
    let count: f64 = points.len() as f64;
    let origin: DVec3 = points.iter().fold(DVec3::ZERO, |sum, point| sum + *point) / count;
    let mut covariance: [[f64; 3]; 3] = [[0.0; 3]; 3];

    for point in points {
      let offset: DVec3 = *point - origin;

      for (row, value) in covariance.iter_mut().zip(offset.to_array()) {
        for (cell, other) in row.iter_mut().zip(offset.to_array()) {
          *cell += value * other;
        }
      }
    }

    let eigen: SymmetricEigen = SymmetricEigen::of(covariance);
    let normal: DVec3 = Self::normalize(eigen.vectors[eigen.get_least()])?;
    let up_hint: DVec3 = if normal.y.abs() > 0.99 { DVec3::X } else { DVec3::Y };
    // `build_camera(from, from - normal, up)`: looking along -normal, right is up × look and up look × right.
    let look: DVec3 = -normal;
    let up: DVec3 = Self::normalize(normal.cross(Self::normalize(up_hint.cross(normal))?))?;
    let right: DVec3 = Self::normalize(up.cross(look))?;
    let up: DVec3 = Self::normalize(look.cross(right))?;

    Some(PortalPlane { origin, right, up })
  }

  /// `vector` divided by its length, as the editor normalizes; none for a vector of no length.
  fn normalize(vector: DVec3) -> Option<DVec3> {
    let length: f64 = vector.length();

    (length > f64::EPSILON).then(|| vector / length)
  }

  /// `SimplifyVertices` (`portal.cpp:307`): of each edge, the point its two neighbouring edges meet at and the area of
  /// the triangle it adds; the edge of the least area collapses into its point. False where no neighbouring edges
  /// meet.
  fn collapse_smallest_edge(vertices: &mut Vec<DVec2>) -> bool {
    let count: usize = vertices.len();
    let mut smallest: Option<(usize, f64, DVec2)> = None;

    for index in 0..count {
      let before: DVec2 = vertices[(index + count - 1) % count];
      let (first, second) = (vertices[index], vertices[(index + 1) % count]);
      let after: DVec2 = vertices[(index + 2) % count];
      let Some(meet) = Self::intersect(before, first, after, second) else {
        continue;
      };
      let area: f64 = Self::get_area(first, meet, second);

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
  fn intersect(a: DVec2, b: DVec2, c: DVec2, d: DVec2) -> Option<DVec2> {
    let (a1, b1, c1) = (b.y - a.y, a.x - b.x, b.x * a.y - a.x * b.y);
    let (a2, b2, c2) = (d.y - c.y, c.x - d.x, d.x * c.y - c.x * d.y);
    let denominator: f64 = a1 * b2 - a2 * b1;

    if denominator.abs() < f64::from(xrf_math::EPS) {
      return None;
    }

    Some(DVec2::new(
      (b1 * c2 - b2 * c1) / denominator,
      (a2 * c1 - a1 * c2) / denominator,
    ))
  }

  /// `tri_area`: Heron's formula.
  fn get_area(a: DVec2, b: DVec2, c: DVec2) -> f64 {
    let distance = |p: DVec2, q: DVec2| (p.x - q.x).hypot(p.y - q.y);
    let (first, second, third) = (distance(a, b), distance(b, c), distance(c, a));
    let half: f64 = (first + second + third) / 2.0;

    (half * (half - first) * (half - second) * (half - third)).sqrt()
  }

  /// The way a polygon winds, as `CPortal::Update` takes it: the sum of each consecutive pair's turn about the middle
  /// (`mknormal`), the last corner not joined back to the first.
  fn get_normal(vertices: &[Vector3d]) -> DVec3 {
    let points: Vec<DVec3> = vertices.iter().map(|vertex| Vec3::from(vertex).as_dvec3()).collect();
    #[expect(clippy::cast_precision_loss, reason = "a portal holds a few dozen corners")]
    let centre: DVec3 = points.iter().fold(DVec3::ZERO, |sum, point| sum + *point) / points.len() as f64;

    points.windows(2).fold(DVec3::ZERO, |sum, pair| {
      Self::normalize((pair[0] - centre).cross(pair[1] - centre)).map_or(sum, |turn| sum + turn)
    })
  }
}
