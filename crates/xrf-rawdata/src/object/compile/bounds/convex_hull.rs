use std::collections::HashMap;
use std::collections::hash_map::Entry;

use glam::DVec3;

/// The convex hull of a point set, by quickhull, in double precision: its faces as triangles over the points' indices,
/// counter-clockwise seen from outside.
///
/// It starts from the widest tetrahedron the points give, then repeatedly takes, of the first face in order that still
/// has points outside it, the point farthest out, removes every face that point sees, found by walking neighbours from
/// that face so the hull stays a closed manifold, and closes the hole with a fan of faces from the point to the horizon.
/// Points within one tolerance of a face, a billionth of the points' extent, count as inside; ties go to the lower
/// index, so one input gives one hull.
pub(crate) struct ConvexHull {
  pub(crate) faces: Vec<[usize; 3]>,
}

/// One face while the hull is built.
struct HullFace {
  corners: [usize; 3],
  /// Its unit outward normal, and its plane's offset along it.
  normal: DVec3,
  offset: f64,
  /// Points outside it, not yet on the hull.
  outside: Vec<usize>,
  is_alive: bool,
}

impl HullFace {
  fn new(points: &[DVec3], corners: [usize; 3]) -> Self {
    let [a, b, c] = corners.map(|it| points[it]);
    let normal: DVec3 = (b - a).cross(c - a).normalize_or_zero();

    Self {
      corners,
      normal,
      offset: normal.dot(a),
      outside: Vec::new(),
      is_alive: true,
    }
  }

  fn distance(&self, point: DVec3) -> f64 {
    self.normal.dot(point) - self.offset
  }

  fn edges(&self) -> [(usize, usize); 3] {
    let [a, b, c] = self.corners;

    [(a, b), (b, c), (c, a)]
  }
}

impl ConvexHull {
  /// The tolerance against the points' extent.
  const RELATIVE_TOLERANCE: f64 = 1e-9;

  /// The hull of `points`, or nothing when they span no volume beyond the tolerance.
  pub(crate) fn of(points: &[DVec3]) -> Option<Self> {
    if points.len() < 4 {
      return None;
    }

    let low: DVec3 = points.iter().fold(DVec3::MAX, |low, point| low.min(*point));
    let high: DVec3 = points.iter().fold(DVec3::MIN, |high, point| high.max(*point));
    // A millimetre floor keeps a tolerance for points gathered at one place.
    let tolerance: f64 = Self::RELATIVE_TOLERANCE * (high - low).max_element().max(1e-3);
    let mut faces: Vec<HullFace> = Self::initial_tetrahedron(points, tolerance)?
      .into_iter()
      .map(|corners| HullFace::new(points, corners))
      .collect();
    let corners: Vec<usize> = faces.iter().flat_map(|face| face.corners).collect();

    for (index, point) in points.iter().enumerate() {
      if corners.contains(&index) {
        continue;
      }

      if let Some(face) = faces.iter_mut().find(|face| face.distance(*point) > tolerance) {
        face.outside.push(index);
      }
    }

    // Each directed edge to the face it bounds: a face's neighbour across an edge owns the reverse edge.
    let mut owners: HashMap<(usize, usize), usize> = HashMap::new();

    for (index, face) in faces.iter().enumerate() {
      for edge in face.edges() {
        owners.insert(edge, index);
      }
    }

    while let Some(start) = faces.iter().position(|face| face.is_alive && !face.outside.is_empty()) {
      Self::add_point(points, tolerance, start, &mut faces, &mut owners);
    }

    Some(Self {
      faces: faces
        .iter()
        .filter(|face| face.is_alive)
        .map(|face| face.corners)
        .collect(),
    })
  }

  /// The indices of the points on the hull, ascending.
  pub(crate) fn vertices(&self) -> Vec<usize> {
    let mut vertices: Vec<usize> = self.faces.iter().flatten().copied().collect();

    vertices.sort_unstable();
    vertices.dedup();
    vertices
  }

  /// Puts the point farthest outside face `start` on the hull.
  fn add_point(
    points: &[DVec3],
    tolerance: f64,
    start: usize,
    faces: &mut Vec<HullFace>,
    owners: &mut HashMap<(usize, usize), usize>,
  ) {
    let eye: usize = *faces[start]
      .outside
      .iter()
      .max_by(|a, b| {
        let face: &HullFace = &faces[start];

        face
          .distance(points[**a])
          .total_cmp(&face.distance(points[**b]))
          .then(b.cmp(a))
      })
      .expect("the face has points outside");

    // The faces the eye sees, connected to the one it lies outside of, and what the walk learnt of their neighbours.
    let mut visible: Vec<usize> = vec![start];
    let mut sees: HashMap<usize, bool> = HashMap::from([(start, true)]);
    let mut next: usize = 0;

    while let Some(face) = visible.get(next).copied() {
      next += 1;

      for (a, b) in faces[face].edges() {
        let neighbour: usize = owners[&(b, a)];

        if let Entry::Vacant(entry) = sees.entry(neighbour) {
          let is_seen: bool = faces[neighbour].distance(points[eye]) > tolerance;

          entry.insert(is_seen);

          if is_seen {
            visible.push(neighbour);
          }
        }
      }
    }

    // The horizon: the visible region's edges whose other side is not visible, in the visible faces' order.
    let horizon: Vec<(usize, usize)> = visible
      .iter()
      .flat_map(|face| faces[*face].edges())
      .filter(|(a, b)| !sees[&owners[&(*b, *a)]])
      .collect();
    let mut orphans: Vec<usize> = Vec::new();

    for face in &visible {
      faces[*face].is_alive = false;
      orphans.append(&mut faces[*face].outside);

      for edge in faces[*face].edges() {
        owners.remove(&edge);
      }
    }

    let first_new: usize = faces.len();

    for (a, b) in horizon {
      let face: HullFace = HullFace::new(points, [a, b, eye]);

      for edge in face.edges() {
        owners.insert(edge, faces.len());
      }

      faces.push(face);
    }

    for point in orphans.into_iter().filter(|it| *it != eye) {
      if let Some(face) = faces[first_new..]
        .iter_mut()
        .find(|face| face.distance(points[point]) > tolerance)
      {
        face.outside.push(point);
      }
    }
  }

  /// The widest tetrahedron the points give, as four outward faces: the two points farthest apart of the six extremes
  /// along the axes, the point farthest from their line, and the point farthest from that plane.
  fn initial_tetrahedron(points: &[DVec3], tolerance: f64) -> Option<[[usize; 3]; 4]> {
    let mut extremes: Vec<usize> = Vec::with_capacity(6);

    for axis in 0..3 {
      let along = |a: &usize, b: &usize| points[*a][axis].total_cmp(&points[*b][axis]).then(a.cmp(b));

      extremes.push((0..points.len()).min_by(along)?);
      extremes.push((0..points.len()).max_by(along)?);
    }

    let mut widest: (usize, usize, f64) = (0, 0, -1.0);

    for (index, first) in extremes.iter().enumerate() {
      for second in &extremes[index + 1..] {
        let span: f64 = points[*first].distance(points[*second]);

        if span > widest.2 {
          widest = (*first, *second, span);
        }
      }
    }

    let (a, b, span) = widest;

    if span <= tolerance {
      return None;
    }

    let line: DVec3 = (points[b] - points[a]) / span;
    let from_line = |index: usize| line.cross(points[index] - points[a]).length();
    let c: usize = (0..points.len()).max_by(|x, y| from_line(*x).total_cmp(&from_line(*y)).then(y.cmp(x)))?;

    if from_line(c) <= tolerance {
      return None;
    }

    let base: HullFace = HullFace::new(points, [a, b, c]);
    let from_base = |index: usize| base.distance(points[index]).abs();
    let d: usize = (0..points.len()).max_by(|x, y| from_base(*x).total_cmp(&from_base(*y)).then(y.cmp(x)))?;

    if from_base(d) <= tolerance {
      return None;
    }

    // The base faces away from the apex, and each side runs one of its edges the other way.
    let (a, b) = if base.distance(points[d]) > 0.0 { (b, a) } else { (a, b) };

    Some([[a, b, c], [b, a, d], [c, b, d], [a, c, d]])
  }
}
