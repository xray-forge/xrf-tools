use xrf_editable_object::EditableBone;
use xrf_math::{Affine3d, Obb, Vector3d};

use crate::object::compile::skeleton::skeleton_hierarchy::SkeletonHierarchy;
use crate::object::compile::skeleton::skeleton_split::SkeletonSplit;

/// The box the skeleton export stores beside each bone's name: around the vertices whose first bone, the lowest by
/// index, is the bone, taken into the bone's rest space.
///
/// The export fits Wild Magic's smallest box (`ComputeOBB_WML`, `MinBox3`); this fits the box along the bone's own axes
/// instead, which holds the same vertices and is larger where the smallest box would have turned. A bone with fewer
/// than three such vertices gets the empty box `Fobb::invalidate` leaves.
pub(crate) struct BoneBoxes;

impl BoneBoxes {
  /// Each bone's box, in bone order.
  pub(crate) fn of(bones: &[EditableBone], splits: &[SkeletonSplit]) -> Vec<Obb> {
    let into_bone: Vec<Affine3d> = SkeletonHierarchy::of(bones)
      .rest_transforms(bones)
      .iter()
      .map(Affine3d::inverse_rigid)
      .collect();
    let mut points: Vec<Vec<Vector3d>> = vec![Vec::new(); bones.len()];

    for vertex in splits.iter().flat_map(|split| &split.vertices) {
      if let Some(first) = vertex.bones.first()
        && let Some(transform) = into_bone.get(usize::from(first.bone))
      {
        points[usize::from(first.bone)].push(transform.transform_point(&vertex.position));
      }
    }

    points.iter().map(|points| Self::box_around(points)).collect()
  }

  fn box_around(points: &[Vector3d]) -> Obb {
    let empty: Obb = Obb {
      rotation: [Vector3d::X, Vector3d::Y, Vector3d::Z],
      center: Vector3d::ZERO,
      half_extents: Vector3d::ZERO,
    };

    if points.len() < 3 {
      return empty;
    }

    let (low, high): (Vector3d, Vector3d) = points[1..].iter().fold((points[0], points[0]), |(low, high), point| {
      (low.min(point), high.max(point))
    });

    Obb {
      center: low.midpoint(&high),
      half_extents: &(&high - &low) * 0.5,
      ..empty
    }
  }
}
