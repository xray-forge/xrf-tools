use xrf_math::Vector3d;

use crate::cform::level_cform_geometry::LevelCformGeometry;
use crate::cform::level_cform_tracer_node::LevelCformTracerNode;

/// Triangles a leaf holds at most.
const LEAF_TRIANGLES: usize = 4;

/// Triangles a run holds at least to be split beside its sibling rather than after it.
const PARALLEL_TRIANGLES: usize = 1 << 14;

/// Nodes a walk holds at once at most: halving from four billion triangles is 32 levels deep, a sibling waiting at each.
const STACK_DEPTH: usize = 64;

/// What `CDB::TestRayTri` takes a ray parallel to a triangle's plane to be.
const PARALLEL_EPSILON: f32 = 1e-12;

/// The collision form as rays are tested against it, `CObjectSpace::RayTest` over `rqtStatic`: its triangles in a
/// bounding volume hierarchy, each tested from both sides, a ray blocked by the first it meets.
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
        let middle: usize = (end - start) / 2;

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

  /// Whether anything of the form lies along a ray within `range` of its origin.
  pub fn is_blocked(&self, origin: &Vector3d<f32>, direction: &Vector3d<f32>, range: f32) -> bool {
    let origin: [f32; 3] = [origin.x, origin.y, origin.z];
    let direction: [f32; 3] = [direction.x, direction.y, direction.z];
    let inverse: [f32; 3] = direction.map(|it| 1.0 / it);
    let mut stack: [usize; STACK_DEPTH] = [0; STACK_DEPTH];
    let mut depth: usize = usize::from(!self.nodes.is_empty());

    while depth > 0 {
      depth -= 1;

      let index: usize = stack[depth];
      let node: &LevelCformTracerNode = &self.nodes[index];

      if !node.is_reached(&origin, &inverse, range) {
        continue;
      }

      if node.count > 0 {
        let first: usize = node.start as usize;

        if self.triangles[first..first + node.count as usize]
          .iter()
          .any(|triangle| self.is_hit(triangle, &origin, &direction, range))
        {
          return true;
        }
      } else {
        stack[depth] = node.start as usize;
        stack[depth + 1] = index + 1;
        depth += 2;
      }
    }

    false
  }

  /// How many triangles it holds.
  pub fn get_triangle_count(&self) -> usize {
    self.triangles.len()
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

    let middle: usize = entries.len() / 2;
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
