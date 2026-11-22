use xrf_math::glam::{Vec3, Vec3A};

use crate::ray::facing::Facing;
use crate::ray::ray::Ray;
use crate::tracer::face_walk::FaceWalk;
use crate::tracer::trace_hit::TraceHit;
use crate::tracer::tracer::Tracer;
use crate::tracer::xrlc::xrlc_child::XrlcChild;
use crate::tracer::xrlc::xrlc_node::XrlcNode;
use crate::triangle::triangle::Triangle;

/// xrLC's ray model: the tree xrCDB builds with Opcode (`MODEL::build_internal`), walked as `COLLIDER::ray_query`
/// walks it, for the passes that must miss what xrLC misses.
///
/// It answers as testing every face would, except for a ray running exactly along a node's face: a node's box can end
/// a rounding short of a corner it holds, and such a ray misses the node and every face below it, as in xrLC.
/// [`crate::BvhTracer`] finds those faces, and is the faster of the two.
///
/// Faces are visited in the tree's order, each node's first side before its second; a model of one face, which xrCDB
/// refuses to build, has no node here, and its face is tested alone.
pub struct XrlcTracer {
  /// Each face, in the order given.
  pub(crate) triangles: Vec<Triangle>,
  /// The root first.
  pub(crate) nodes: Vec<XrlcNode>,
}

impl XrlcTracer {
  /// Builds the tree over faces given as three indices into `vertices` each.
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

  /// Builds the tree over the triangles given, each face named by its index among them. The tree depends on their
  /// order, as Opcode's does on the model's.
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

    Self {
      nodes: Self::build_nodes(&triangles),
      triangles,
    }
  }
}

impl FaceWalk for XrlcTracer {
  fn get_triangles(&self) -> &[Triangle] {
    &self.triangles
  }

  /// `_stab`, its box test `_box_sse` with the reach the query's `rRange`: a node is tested when the walk comes to it,
  /// so a hit in its first side narrows the test of its second.
  fn walk(&self, ray: &Ray, mut visit: impl FnMut(u32, &mut f32) -> bool) {
    let mut reach: f32 = ray.range;

    if self.nodes.is_empty() {
      if !self.triangles.is_empty() {
        visit(0, &mut reach);
      }

      return;
    }

    // `ray_collider::_init`: each component's inverse as single precision division gives it, a zero's infinite.
    let inverse: Vec3A = Vec3A::ONE / ray.direction;
    let mut stack: Vec<XrlcChild> = Vec::with_capacity(64);

    stack.push(XrlcChild::Node(0));

    while let Some(child) = stack.pop() {
      match child {
        XrlcChild::Face(face) => {
          if visit(face, &mut reach) {
            return;
          }
        }
        XrlcChild::Node(index) => {
          let node: &XrlcNode = &self.nodes[index as usize];

          if node.get_entry(ray.origin, inverse).is_none_or(|entry| entry > reach) {
            continue;
          }

          stack.push(node.children[1]);
          stack.push(node.children[0]);
        }
      }
    }
  }
}

impl Tracer for XrlcTracer {
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
    for (index, ray) in rays.iter().enumerate() {
      self.for_each_hit(ray, |hit| visit(index, hit));
    }
  }

  fn for_each_face_in_box(&self, center: Vec3, half: Vec3, mut visit: impl FnMut(u32)) {
    if self.nodes.is_empty() {
      if self
        .triangles
        .first()
        .is_some_and(|triangle| triangle.overlaps_box(center, half))
      {
        visit(0);
      }

      return;
    }

    let (low, high) = (Vec3A::from(center - half), Vec3A::from(center + half));
    let mut stack: Vec<XrlcChild> = Vec::with_capacity(64);

    stack.push(XrlcChild::Node(0));

    while let Some(child) = stack.pop() {
      match child {
        XrlcChild::Face(face) => {
          if self.triangles[face as usize].overlaps_box(center, half) {
            visit(face);
          }
        }
        XrlcChild::Node(index) => {
          let node: &XrlcNode = &self.nodes[index as usize];

          if node.low.cmpgt(high).any() || node.high.cmplt(low).any() {
            continue;
          }

          stack.push(node.children[1]);
          stack.push(node.children[0]);
        }
      }
    }
  }

  /// `_prim` under `bNearest`: a nearer hit replaces the kept one and shortens the reach the boxes are tested within.
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
