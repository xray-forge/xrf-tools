use std::time::Duration;

use glam::{Vec3, Vec3A};
use obvhs::BvhBuildParams;
use obvhs::bvh2::Bvh2;
use obvhs::bvh2::builder::build_bvh2_from_tris;
use obvhs::bvh2::node::Bvh2Node;

use crate::ray::facing::Facing;
use crate::ray::ray::Ray;
use crate::ray::slab_ray::SlabRay;
use crate::tracer::face_walk::FaceWalk;
use crate::tracer::trace_hit::TraceHit;
use crate::tracer::tracer::Tracer;
use crate::triangle::triangle::Triangle;

/// The triangles in a binary bounding volume hierarchy, `obvhs`' PLOC build at its medium quality, walked by its own
/// traversal: `obvhs`' own takes a zero direction component's inverse as a large finite number and misses faces on such
/// rays, where `SlabRay` takes it as infinite.
pub struct BvhTracer {
  /// Each face, in the order given.
  pub(crate) triangles: Vec<Triangle>,
  /// The root first; an inner node's children side by side from its `first_index`.
  pub(crate) nodes: Vec<Bvh2Node>,
  /// A leaf's faces, from its `first_index` on.
  pub(crate) faces: Vec<u32>,
}

impl BvhTracer {
  /// Rays a packet's masks hold; [`Tracer::for_each_packet_hit`] traces more as several packets.
  pub const PACKET_RAYS: usize = 256;

  /// The `u64` words of a packet's masks.
  pub(crate) const MASK_WORDS: usize = Self::PACKET_RAYS / 64;

  /// Builds the hierarchy over faces given as three indices into `vertices` each.
  ///
  /// # Panics
  ///
  /// Panics when a face names a vertex `vertices` does not hold, or as [`Self::of_triangles`] does.
  pub fn new(vertices: &[Vec3], faces: &[[u32; 3]]) -> Self {
    Self::of_triangles(
      faces
        .iter()
        .map(|face| Triangle::new(face.map(|index| vertices[index as usize])))
        .collect(),
    )
  }

  /// Builds the hierarchy over the triangles given, each face named by its index among them.
  ///
  /// # Panics
  ///
  /// Panics when the triangles are more than a `u32` face index names.
  pub fn of_triangles(triangles: Vec<Triangle>) -> Self {
    assert!(
      u32::try_from(triangles.len()).is_ok(),
      "a tracer names its faces by u32, and {} triangles are more",
      triangles.len()
    );

    let shapes: Vec<obvhs::triangle::Triangle> = triangles
      .iter()
      .map(|triangle| obvhs::triangle::Triangle {
        v0: triangle.corners[0],
        v1: triangle.corners[1],
        v2: triangle.corners[2],
      })
      .collect();
    let mut bvh: Bvh2 = build_bvh2_from_tris(&shapes, BvhBuildParams::medium_build(), &mut Duration::default());

    // Only the nodes and the leaves' faces are walked; the builder's other tables would only hold memory.
    Self {
      triangles,
      nodes: std::mem::take(&mut bvh.nodes),
      faces: std::mem::take(&mut bvh.primitive_indices),
    }
  }

  /// A leaf's faces.
  pub(crate) fn get_leaf_faces(&self, node: &Bvh2Node) -> &[u32] {
    &self.faces[node.first_index as usize..(node.first_index + node.prim_count) as usize]
  }
}

impl FaceWalk for BvhTracer {
  fn get_triangles(&self) -> &[Triangle] {
    &self.triangles
  }

  /// Visits each face in a leaf a ray reaches within a reach the visitor may shorten, until the visitor returns true.
  fn walk(&self, ray: &Ray, mut visit: impl FnMut(u32, &mut f32) -> bool) {
    if self.nodes.is_empty() {
      return;
    }

    let slab: SlabRay = SlabRay::new(ray.origin, ray.direction);
    let mut reach: f32 = ray.range;
    let mut stack: Vec<u32> = Vec::with_capacity(64);

    stack.push(0);

    while let Some(index) = stack.pop() {
      let node: &Bvh2Node = &self.nodes[index as usize];

      if !slab.reaches(reach, node.aabb.min, node.aabb.max) {
        continue;
      }

      if node.is_leaf() {
        for face in self.get_leaf_faces(node) {
          if visit(*face, &mut reach) {
            return;
          }
        }
      } else {
        stack.push(node.first_index + 1);
        stack.push(node.first_index);
      }
    }
  }
}

impl Tracer for BvhTracer {
  fn get_face_count(&self) -> usize {
    self.triangles.len()
  }

  fn get_triangle(&self, face: u32) -> Triangle {
    self.triangles[face as usize]
  }

  fn for_each_hit_facing(&self, ray: &Ray, facing: Facing, visit: impl FnMut(TraceHit)) {
    self.walk_hits(ray, facing, visit);
  }

  fn for_each_packet_hit(&self, rays: &[Ray], mut visit: impl FnMut(usize, TraceHit)) {
    for (packet, first) in rays.chunks(Self::PACKET_RAYS).zip((0..).step_by(Self::PACKET_RAYS)) {
      self.trace_packet(packet, |index, hit| visit(first + index, hit));
    }
  }

  fn for_each_face_in_box(&self, center: Vec3, half: Vec3, mut visit: impl FnMut(u32)) {
    if self.nodes.is_empty() {
      return;
    }

    let (low, high) = (Vec3A::from(center - half), Vec3A::from(center + half));
    let mut stack: Vec<u32> = Vec::with_capacity(64);

    stack.push(0);

    while let Some(index) = stack.pop() {
      let node: &Bvh2Node = &self.nodes[index as usize];

      if node.aabb.min.cmpgt(high).any() || node.aabb.max.cmplt(low).any() {
        continue;
      }

      if node.is_leaf() {
        for face in self.get_leaf_faces(node) {
          if self.triangles[*face as usize].overlaps_box(center, half) {
            visit(*face);
          }
        }
      } else {
        stack.push(node.first_index + 1);
        stack.push(node.first_index);
      }
    }
  }

  fn get_nearest_hit(&self, ray: &Ray) -> Option<TraceHit> {
    self.walk_nearest_hit(ray)
  }

  fn is_blocked(&self, ray: &Ray) -> bool {
    self.walk_is_blocked(ray)
  }

  fn get_face_nearest_along(&self, point: Vec3, direction: Vec3, reach: f32) -> Option<u32> {
    self.walk_face_nearest_along(point, direction, reach)
  }
}
