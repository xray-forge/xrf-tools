use xrf_math::Vector3d;

use crate::cform::level_cform_geometry::LevelCformGeometry;
use crate::cform::level_cform_hit::LevelCformHit;
use crate::cform::level_cform_tracer_node::LevelCformTracerNode;

/// Triangles a leaf holds at most.
const LEAF_TRIANGLES: usize = 4;

/// Triangles a run holds at least to be split beside its sibling rather than after it.
const PARALLEL_TRIANGLES: usize = 1 << 14;

/// Nodes a walk holds at once at most: halving from four billion triangles is 32 levels deep, a sibling waiting at each.
const STACK_DEPTH: usize = 64;

/// What `CDB::TestRayTri` takes a ray parallel to a triangle's plane to be.
const PARALLEL_EPSILON: f32 = 1e-12;

/// `EPS`: the least determinant `CDB`'s culled ray test takes a front face to have.
const FRONT_EPSILON: f32 = xrf_math::EPS;

/// The collision form's triangles in a bounding volume hierarchy, for the engine's static ray tests and picks.
pub struct LevelCformTracer {
  vertices: Vec<[f32; 3]>,
  /// Each triangle's corners, in the order the leaves hold them.
  triangles: Vec<[u32; 3]>,
  /// Depth first, the root first, each inner node's first child right after it.
  nodes: Vec<LevelCformTracerNode>,
}

impl LevelCformTracer {
  /// Builds the hierarchy over every face of a collision form, each branch split at its middle triangle along the axis
  /// its triangles' centres spread furthest. The splits always fall at the middle, so the triangles are ordered first,
  /// sibling runs side by side, and the nodes laid out after from the counts alone.
  pub fn new(geometry: &LevelCformGeometry) -> Self {
    let vertices: Vec<[f32; 3]> = geometry.get_vertices().iter().map(|it| [it.x, it.y, it.z]).collect();
    // Each triangle beside its centre, which the splits order it by.
    let mut entries: Vec<([u32; 3], [f32; 3])> = geometry
      .get_faces()
      .iter()
      .map(|face| {
        let [a, b, c] = face.vertices.map(|index| vertices[index as usize]);

        (face.vertices, [0, 1, 2].map(|axis| (a[axis] + b[axis] + c[axis]) / 3.0))
      })
      .collect();

    Self::order(&mut entries);

    let mut nodes: Vec<LevelCformTracerNode> = Vec::with_capacity(entries.len() / LEAF_TRIANGLES * 2 + 1);
    // Runs still to be laid out, and the node waiting to be told where its second child went. Taken first child first,
    // so each inner node's first child is the node laid out right after it.
    let mut pending: Vec<(usize, usize, Option<usize>)> = Vec::new();

    if !entries.is_empty() {
      pending.push((0, entries.len(), None));
    }

    while let Some((start, end, parent)) = pending.pop() {
      let index: usize = nodes.len();

      if let Some(parent) = parent {
        nodes[parent].start = index as u32;
      }

      if end - start > LEAF_TRIANGLES {
        let middle: usize = split(end - start);

        nodes.push(LevelCformTracerNode::inner());
        pending.push((start + middle, end, Some(index)));
        pending.push((start, start + middle, None));
      } else {
        nodes.push(LevelCformTracerNode::leaf(start as u32, (end - start) as u32));
      }
    }

    let triangles: Vec<[u32; 3]> = entries.into_iter().map(|(triangle, _)| triangle).collect();

    Self::bound(&mut nodes, &vertices, &triangles);

    Self {
      nodes,
      triangles,
      vertices,
    }
  }

  /// Whether anything of the form lies along a ray within `range` of its origin, from either side.
  pub fn is_blocked(&self, origin: &Vector3d<f32>, direction: &Vector3d<f32>, range: f32) -> bool {
    let origin: [f32; 3] = [origin.x, origin.y, origin.z];
    let direction: [f32; 3] = [direction.x, direction.y, direction.z];
    let mut is_blocked: bool = false;

    self.walk(&origin, &direction, range, |triangle, _| {
      is_blocked = self.is_hit(triangle, &origin, &direction, range);
      is_blocked
    });

    is_blocked
  }

  /// `CObjectSpace::RayPick` over `rqtStatic` (`OPT_ONLYNEAREST | OPT_CULL`): the nearest front face a ray meets.
  pub fn get_nearest_hit(
    &self,
    origin: &Vector3d<f32>,
    direction: &Vector3d<f32>,
    range: f32,
  ) -> Option<LevelCformHit> {
    let origin: [f32; 3] = [origin.x, origin.y, origin.z];
    let direction: [f32; 3] = [direction.x, direction.y, direction.z];
    let mut nearest: Option<(f32, [u32; 3])> = None;

    self.walk(&origin, &direction, range, |triangle, reach| {
      if let Some(distance) = self.get_front_distance(triangle, &origin, &direction)
        && distance > 0.0
        && distance <= *reach
        && nearest.is_none_or(|(best, _)| distance < best)
      {
        nearest = Some((distance, *triangle));
        *reach = distance;
      }

      false
    });

    nearest.map(|(distance, triangle)| {
      let [a, b, c] = triangle.map(|index| self.vertices[index as usize]);
      let normal: [f32; 3] = cross(&sub(&b, &a), &sub(&c, &b));
      let length_sqr: f32 = dot(&normal, &normal);
      // `normalize_safe`: unchanged when too short to scale.
      let scale: f32 = if length_sqr > f32::MIN_POSITIVE {
        (1.0 / length_sqr).sqrt()
      } else {
        1.0
      };

      LevelCformHit {
        distance,
        normal: Vector3d::new(normal[0] * scale, normal[1] * scale, normal[2] * scale),
      }
    })
  }

  /// How many triangles it holds.
  pub fn get_triangle_count(&self) -> usize {
    self.triangles.len()
  }

  /// Visits each triangle in a leaf the ray reaches within a reach the visitor may shorten, until it returns true.
  fn walk(
    &self,
    origin: &[f32; 3],
    direction: &[f32; 3],
    range: f32,
    mut visit: impl FnMut(&[u32; 3], &mut f32) -> bool,
  ) {
    let inverse: [f32; 3] = direction.map(|it| 1.0 / it);
    let mut stack: [usize; STACK_DEPTH] = [0; STACK_DEPTH];
    let mut depth: usize = usize::from(!self.nodes.is_empty());
    let mut reach: f32 = range;

    while depth > 0 {
      depth -= 1;

      let index: usize = stack[depth];
      let node: &LevelCformTracerNode = &self.nodes[index];

      if !node.is_reached(origin, &inverse, reach) {
        continue;
      }

      if node.count > 0 {
        let first: usize = node.start as usize;

        for triangle in &self.triangles[first..first + node.count as usize] {
          if visit(triangle, &mut reach) {
            return;
          }
        }
      } else {
        stack[depth] = node.start as usize;
        stack[depth + 1] = index + 1;
        depth += 2;
      }
    }
  }

  /// `CDB::TestRayTri` without culling: whether a ray crosses a triangle from either side within `range`.
  fn is_hit(&self, triangle: &[u32; 3], origin: &[f32; 3], direction: &[f32; 3], range: f32) -> bool {
    let [a, b, c] = triangle.map(|index| self.vertices[index as usize]);
    let first: [f32; 3] = sub(&b, &a);
    let second: [f32; 3] = sub(&c, &a);
    let across: [f32; 3] = cross(direction, &second);
    let determinant: f32 = dot(&first, &across);

    if determinant.abs() < PARALLEL_EPSILON {
      return false;
    }

    let inverse: f32 = 1.0 / determinant;
    let offset: [f32; 3] = sub(origin, &a);
    let u: f32 = dot(&offset, &across) * inverse;

    if !(0.0..=1.0).contains(&u) {
      return false;
    }

    let up: [f32; 3] = cross(&offset, &first);
    let v: f32 = dot(direction, &up) * inverse;

    if v < 0.0 || u + v > 1.0 {
      return false;
    }

    let distance: f32 = dot(&second, &up) * inverse;

    distance > 0.0 && distance < range
  }

  /// `RayCollider::_tri` with culling: how far along a ray it meets a triangle's front side, if it does.
  fn get_front_distance(&self, triangle: &[u32; 3], origin: &[f32; 3], direction: &[f32; 3]) -> Option<f32> {
    let [a, b, c] = triangle.map(|index| self.vertices[index as usize]);
    let first: [f32; 3] = sub(&b, &a);
    let second: [f32; 3] = sub(&c, &a);
    let across: [f32; 3] = cross(direction, &second);
    let determinant: f32 = dot(&first, &across);

    if determinant < FRONT_EPSILON {
      return None;
    }

    let offset: [f32; 3] = sub(origin, &a);
    let u: f32 = dot(&offset, &across);

    if u < 0.0 || u > determinant {
      return None;
    }

    let up: [f32; 3] = cross(&offset, &first);
    let v: f32 = dot(direction, &up);

    if v < 0.0 || u + v > determinant {
      return None;
    }

    Some(dot(&second, &up) * (1.0 / determinant))
  }

  /// Every node's box, from the last up: a leaf's around its triangles, an inner node's around its children's, which
  /// come after it. Each triangle is read once.
  fn bound(nodes: &mut [LevelCformTracerNode], vertices: &[[f32; 3]], triangles: &[[u32; 3]]) {
    for index in (0..nodes.len()).rev() {
      let node: LevelCformTracerNode = nodes[index];

      if node.count > 0 {
        let first: usize = node.start as usize;

        for triangle in &triangles[first..first + node.count as usize] {
          for vertex in triangle {
            nodes[index].enclose(&vertices[*vertex as usize], &vertices[*vertex as usize]);
          }
        }
      } else {
        let [first, second] = [nodes[index + 1], nodes[node.start as usize]];

        nodes[index].enclose(&first.min, &first.max);
        nodes[index].enclose(&second.min, &second.max);
      }
    }
  }

  /// Orders a run as its splits divide it: the half with the smaller centres along its widest axis first, each half
  /// ordered the same way.
  fn order(entries: &mut [([u32; 3], [f32; 3])]) {
    if entries.len() <= LEAF_TRIANGLES {
      return;
    }

    let middle: usize = split(entries.len());
    let axis: usize = Self::widest_axis(entries);

    entries.select_nth_unstable_by(middle, |first, second| first.1[axis].total_cmp(&second.1[axis]));

    let (first, second) = entries.split_at_mut(middle);

    if first.len() + second.len() >= PARALLEL_TRIANGLES {
      rayon::join(|| Self::order(first), || Self::order(second));
    } else {
      Self::order(first);
      Self::order(second);
    }
  }

  /// The axis the centres of a run spread furthest along.
  fn widest_axis(entries: &[([u32; 3], [f32; 3])]) -> usize {
    let mut min: [f32; 3] = [f32::INFINITY; 3];
    let mut max: [f32; 3] = [f32::NEG_INFINITY; 3];

    for (_, centre) in entries {
      for axis in 0..3 {
        min[axis] = min[axis].min(centre[axis]);
        max[axis] = max[axis].max(centre[axis]);
      }
    }

    (0..3)
      .max_by(|first, second| (max[*first] - min[*first]).total_cmp(&(max[*second] - min[*second])))
      .unwrap_or(0)
  }
}

/// Where a run of triangles is split, which both the ordering and the layout go by, so the tree matches the order.
fn split(count: usize) -> usize {
  count / 2
}

fn sub(first: &[f32; 3], second: &[f32; 3]) -> [f32; 3] {
  [first[0] - second[0], first[1] - second[1], first[2] - second[2]]
}

fn cross(first: &[f32; 3], second: &[f32; 3]) -> [f32; 3] {
  [
    first[1] * second[2] - first[2] * second[1],
    first[2] * second[0] - first[0] * second[2],
    first[0] * second[1] - first[1] * second[0],
  ]
}

fn dot(first: &[f32; 3], second: &[f32; 3]) -> f32 {
  first[0] * second[0] + first[1] * second[1] + first[2] * second[2]
}
