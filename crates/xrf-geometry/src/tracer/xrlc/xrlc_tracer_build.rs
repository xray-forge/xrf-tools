use std::ops::Range;

use xrf_math::glam::Vec3;

use crate::tracer::xrlc::xrlc_child::XrlcChild;
use crate::tracer::xrlc::xrlc_node::XrlcNode;
use crate::tracer::xrlc::xrlc_tracer::XrlcTracer;
use crate::triangle::triangle::Triangle;

/// A run of the face list a node is built over, and where its index goes once it has one.
struct Pending {
  run: Range<usize>,
  /// The side naming it; none for the root.
  parent: Option<PendingParent>,
}

/// The side of a node a pending node hangs from.
struct PendingParent {
  node: u32,
  side: usize,
}

impl XrlcTracer {
  /// `1 / 3` as Opcode writes it (`INV3`), in single precision.
  const THIRD: f32 = 1.0 / 3.0;

  /// The nodes of Opcode's complete tree over `triangles` as `MODEL::build_internal` asks for it, no leaves and not
  /// quantized: the root first, each node's first side and all below it before its second (`_BuildNoLeafTree`).
  ///
  /// Built without recursion, as a split can leave one face a side all the way down.
  pub(crate) fn build_nodes(triangles: &[Triangle]) -> Vec<XrlcNode> {
    if triangles.len() < 2 {
      return Vec::new();
    }

    let corners: Vec<[Vec3; 3]> = triangles
      .iter()
      .map(|triangle| triangle.corners.map(Vec3::from))
      .collect();
    let mut faces: Vec<u32> = (0..triangles.len() as u32).collect();
    let mut nodes: Vec<XrlcNode> = Vec::with_capacity(triangles.len() - 1);
    let mut pending: Vec<Pending> = vec![Pending {
      run: 0..faces.len(),
      parent: None,
    }];

    while let Some(Pending { run, parent }) = pending.pop() {
      let part: &mut [u32] = &mut faces[run.clone()];
      let (low, high) = Self::get_bounds(&corners, part);
      let positive: usize = Self::split(&corners, part, (high + low) * 0.5);
      let index: u32 = nodes.len() as u32;
      let sides: [Range<usize>; 2] = [run.start..run.start + positive, run.start + positive..run.end];
      let children: [XrlcChild; 2] = sides.clone().map(|side| {
        if side.len() == 1 {
          XrlcChild::Face(faces[side.start])
        } else {
          // Numbered once walked to.
          XrlcChild::Node(u32::MAX)
        }
      });

      nodes.push(XrlcNode::new(low, high, children));

      if let Some(PendingParent { node, side }) = parent {
        nodes[node as usize].children[side] = XrlcChild::Node(index);
      }

      // The second side waits under the first, so the first and everything below it are numbered before it.
      for (side, run) in sides.into_iter().enumerate().rev() {
        if run.len() > 1 {
          pending.push(Pending {
            run,
            parent: Some(PendingParent { node: index, side }),
          });
        }
      }
    }

    nodes
  }

  /// The least and the greatest corner of a node's faces (`AABBTreeOfTrianglesBuilder::ComputeGlobalBox`).
  fn get_bounds(corners: &[[Vec3; 3]], faces: &[u32]) -> (Vec3, Vec3) {
    faces.iter().flat_map(|face| corners[*face as usize]).fold(
      (Vec3::splat(f32::MAX), Vec3::splat(-f32::MAX)),
      |(low, high), corner| (low.min(corner), high.max(corner)),
    )
  }

  /// A face's centre on `axis` (`AABBTreeOfTrianglesBuilder::GetSplittingValue`).
  fn get_center(corners: &[[Vec3; 3]], face: u32, axis: usize) -> f32 {
    let [first, second, third] = corners[face as usize];

    (first[axis] + second[axis] + third[axis]) * Self::THIRD
  }

  /// Reorders a node's faces on the axis their centres vary most along, those whose centre lies above `center` first,
  /// and returns how many those are, or half the faces where all fall on one side (`AABBTreeNode::Subdivide` under
  /// `SPLIT_SPLATTERPOINTS`).
  ///
  /// The split is the box middle, though xrCDB asks for `SPLIT_GEOMCENTER`: the triangles' builder misnames its
  /// override `GetSplittingValue`, so `AABBTreeBuilder::GetSplittingValueEx` answers with the base builder's middle.
  fn split(corners: &[[Vec3; 3]], faces: &mut [u32], center: Vec3) -> usize {
    let count: usize = faces.len();
    let mut means: [f32; 3] = [0.0; 3];

    for face in faces.iter() {
      for (axis, mean) in means.iter_mut().enumerate() {
        *mean += Self::get_center(corners, *face, axis);
      }
    }

    // `Point::operator/=` multiplies by the inverse.
    let inverse: f32 = 1.0 / count as f32;
    let means: [f32; 3] = means.map(|mean| mean * inverse);
    let mut variances: [f32; 3] = [0.0; 3];

    for face in faces.iter() {
      for (axis, variance) in variances.iter_mut().enumerate() {
        let off: f32 = Self::get_center(corners, *face, axis) - means[axis];

        *variance += off * off;
      }
    }

    let inverse: f32 = 1.0 / (count - 1) as f32;
    let variances: [f32; 3] = variances.map(|variance| variance * inverse);
    // `Point::LargestAxis`: a later axis only where strictly greater.
    let mut axis: usize = 0;

    for other in [1, 2] {
      if variances[other] > variances[axis] {
        axis = other;
      }
    }

    let value: f32 = center[axis];
    let mut positive: usize = 0;

    for index in 0..count {
      if Self::get_center(corners, faces[index], axis) > value {
        faces.swap(index, positive);
        positive += 1;
      }
    }

    if positive == 0 || positive == count {
      count >> 1
    } else {
      positive
    }
  }
}
