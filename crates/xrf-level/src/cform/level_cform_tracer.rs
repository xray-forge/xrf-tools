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

/// A triangle as the hierarchy is built: its corners, its centre, which the splits order it by, and its material.
type TracerEntry = ([u32; 3], [f32; 3], u16);

/// The collision form's triangles in a bounding volume hierarchy, for the engine's static ray tests and picks.
pub struct LevelCformTracer {
  vertices: Vec<Vector3d<f32>>,
  /// Each triangle's corners, in the order the leaves hold them.
  triangles: Vec<[u32; 3]>,
  /// Each triangle's game material, by its `gamemtl.xr` id, in the same order.
  materials: Vec<u16>,
  /// Depth first, the root first, each inner node's first child right after it.
  nodes: Vec<LevelCformTracerNode>,
}

impl LevelCformTracer {
  /// Builds the hierarchy over every face of a collision form, each branch split at its middle triangle along the axis
  /// its triangles' centres spread furthest. The splits always fall at the middle, so the triangles are ordered first,
  /// sibling runs side by side, and the nodes laid out after from the counts alone.
  pub fn new(geometry: &LevelCformGeometry) -> Self {
    let vertices: Vec<Vector3d<f32>> = geometry.get_vertices().to_vec();
    // Each triangle beside its centre, which the splits order it by, and its material, which travels with it.
    let mut entries: Vec<TracerEntry> = geometry
      .get_faces()
      .iter()
      .map(|face| {
        let [a, b, c] = face.vertices.map(|index| &vertices[index as usize]);

        (
          face.vertices,
          (&(a + b) + c).to_array().map(|axis| axis / 3.0),
          face.material,
        )
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

    let (triangles, materials): (Vec<[u32; 3]>, Vec<u16>) = entries
      .into_iter()
      .map(|(triangle, _, material)| (triangle, material))
      .unzip();

    Self::bound(&mut nodes, &vertices, &triangles);

    Self {
      nodes,
      triangles,
      materials,
      vertices,
    }
  }

  /// Whether anything of the form lies along a ray within `range` of its origin, from either side.
  pub fn is_blocked(&self, origin: &Vector3d<f32>, direction: &Vector3d<f32>, range: f32) -> bool {
    let mut is_blocked: bool = false;

    self.walk(origin, direction, range, |index, _| {
      is_blocked = self
        .get_distance(index, origin, direction)
        .is_some_and(|distance| distance < range);
      is_blocked
    });

    is_blocked
  }

  /// The game material of the nearest face a ray crosses within `range` of its origin, from either side: what a
  /// surface lying on the form is made of, read along its own normal whichever way its collision faces were wound.
  pub fn get_nearest_material(&self, origin: &Vector3d<f32>, direction: &Vector3d<f32>, range: f32) -> Option<u16> {
    let mut nearest: Option<usize> = None;

    self.walk(origin, direction, range, |index, reach| {
      if let Some(distance) = self.get_distance(index, origin, direction)
        && distance < *reach
      {
        nearest = Some(index);
        *reach = distance;
      }

      false
    });

    nearest.map(|index| self.materials[index])
  }

  /// `CObjectSpace::RayPick` over `rqtStatic` (`OPT_ONLYNEAREST | OPT_CULL`): the nearest front face a ray meets.
  pub fn get_nearest_hit(
    &self,
    origin: &Vector3d<f32>,
    direction: &Vector3d<f32>,
    range: f32,
  ) -> Option<LevelCformHit> {
    let mut nearest: Option<(f32, usize)> = None;

    self.walk(origin, direction, range, |index, reach| {
      if let Some(distance) = self.get_front_distance(&self.triangles[index], origin, direction)
        && distance > 0.0
        && distance <= *reach
        && nearest.is_none_or(|(best, _)| distance < best)
      {
        nearest = Some((distance, index));
        *reach = distance;
      }

      false
    });

    nearest.map(|(distance, index)| {
      let [a, b, c] = self.triangles[index].map(|index| &self.vertices[index as usize]);
      let normal: Vector3d<f32> = (b - a).cross(&(c - b));
      let length_sqr: f32 = normal.length_squared();
      // `normalize_safe`: unchanged when too short to scale.
      let scale: f32 = if length_sqr > f32::MIN_POSITIVE {
        (1.0 / length_sqr).sqrt()
      } else {
        1.0
      };

      LevelCformHit {
        distance,
        normal: &normal * scale,
        material: self.materials[index],
      }
    })
  }

  /// How many triangles it holds.
  pub fn get_triangle_count(&self) -> usize {
    self.triangles.len()
  }

  /// Visits each triangle, by its index, in a leaf the ray reaches within a reach the visitor may shorten, until it
  /// returns true.
  fn walk(
    &self,
    origin: &Vector3d<f32>,
    direction: &Vector3d<f32>,
    range: f32,
    mut visit: impl FnMut(usize, &mut f32) -> bool,
  ) {
    let origin: [f32; 3] = origin.to_array();
    let inverse: [f32; 3] = direction.to_array().map(|it| 1.0 / it);
    let mut stack: [usize; STACK_DEPTH] = [0; STACK_DEPTH];
    let mut depth: usize = usize::from(!self.nodes.is_empty());
    let mut reach: f32 = range;

    while depth > 0 {
      depth -= 1;

      let index: usize = stack[depth];
      let node: &LevelCformTracerNode = &self.nodes[index];

      if !node.is_reached(&origin, &inverse, reach) {
        continue;
      }

      if node.count > 0 {
        let first: usize = node.start as usize;

        for triangle in first..first + node.count as usize {
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

  /// `CDB::TestRayTri` without culling: how far ahead along a ray it crosses a triangle from either side, if it does.
  fn get_distance(&self, index: usize, origin: &Vector3d<f32>, direction: &Vector3d<f32>) -> Option<f32> {
    let [a, b, c] = self.triangles[index].map(|index| &self.vertices[index as usize]);
    let first: Vector3d<f32> = b - a;
    let second: Vector3d<f32> = c - a;
    let across: Vector3d<f32> = direction.cross(&second);
    let determinant: f32 = first.dot(&across);

    if determinant.abs() < PARALLEL_EPSILON {
      return None;
    }

    let inverse: f32 = 1.0 / determinant;
    let offset: Vector3d<f32> = origin - a;
    let u: f32 = offset.dot(&across) * inverse;

    if !(0.0..=1.0).contains(&u) {
      return None;
    }

    let up: Vector3d<f32> = offset.cross(&first);
    let v: f32 = direction.dot(&up) * inverse;

    if v < 0.0 || u + v > 1.0 {
      return None;
    }

    Some(second.dot(&up) * inverse).filter(|distance| *distance > 0.0)
  }

  /// `RayCollider::_tri` with culling: how far along a ray it meets a triangle's front side, if it does.
  fn get_front_distance(&self, triangle: &[u32; 3], origin: &Vector3d<f32>, direction: &Vector3d<f32>) -> Option<f32> {
    let [a, b, c] = triangle.map(|index| &self.vertices[index as usize]);
    let first: Vector3d<f32> = b - a;
    let second: Vector3d<f32> = c - a;
    let across: Vector3d<f32> = direction.cross(&second);
    let determinant: f32 = first.dot(&across);

    if determinant < FRONT_EPSILON {
      return None;
    }

    let offset: Vector3d<f32> = origin - a;
    let u: f32 = offset.dot(&across);

    if u < 0.0 || u > determinant {
      return None;
    }

    let up: Vector3d<f32> = offset.cross(&first);
    let v: f32 = direction.dot(&up);

    if v < 0.0 || u + v > determinant {
      return None;
    }

    Some(second.dot(&up) * (1.0 / determinant))
  }

  /// Every node's box, from the last up: a leaf's around its triangles, an inner node's around its children's, which
  /// come after it. Each triangle is read once.
  fn bound(nodes: &mut [LevelCformTracerNode], vertices: &[Vector3d<f32>], triangles: &[[u32; 3]]) {
    for index in (0..nodes.len()).rev() {
      let node: LevelCformTracerNode = nodes[index];

      if node.count > 0 {
        let first: usize = node.start as usize;

        for triangle in &triangles[first..first + node.count as usize] {
          for vertex in triangle {
            let corner: [f32; 3] = vertices[*vertex as usize].to_array();

            nodes[index].enclose(&corner, &corner);
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
  fn order(entries: &mut [TracerEntry]) {
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
  fn widest_axis(entries: &[TracerEntry]) -> usize {
    let mut min: [f32; 3] = [f32::INFINITY; 3];
    let mut max: [f32; 3] = [f32::NEG_INFINITY; 3];

    for (_, centre, _) in entries {
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
