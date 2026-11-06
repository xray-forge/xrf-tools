use std::collections::HashMap;

use xrf_editable_object::EditableBone;
use xrf_math::{Affine3d, Obb, Vector3d};

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
    let into_bone: Vec<Affine3d> = Self::get_rest_transforms(bones)
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

  /// Each bone's rest transform, `CalculateRest`: its rest rotation (`setXYZi`) and offset under its parent's.
  fn get_rest_transforms(bones: &[EditableBone]) -> Vec<Affine3d> {
    let index_of: HashMap<&str, usize> = bones
      .iter()
      .enumerate()
      .map(|(index, bone)| (bone.definition.name.as_str(), index))
      .collect();
    let mut transforms: Vec<Option<Affine3d>> = vec![None; bones.len()];

    for bone in 0..bones.len() {
      Self::resolve(bone, bones, &index_of, &mut transforms, 0);
    }

    transforms
      .into_iter()
      .map(|it| it.unwrap_or(Affine3d::IDENTITY))
      .collect()
  }

  fn resolve(
    bone: usize,
    bones: &[EditableBone],
    index_of: &HashMap<&str, usize>,
    transforms: &mut [Option<Affine3d>],
    depth: usize,
  ) -> Affine3d {
    if let Some(transform) = transforms[bone] {
      return transform;
    }

    let local: Affine3d =
      Affine3d::from_xyz_inverse(&bones[bone].bind_pose.rotation).with_translation(bones[bone].bind_pose.offset);
    // A parent chain longer than the bones loops; its bones are placed as roots.
    let parent: Affine3d = match index_of.get(bones[bone].definition.parent.as_str()) {
      Some(parent) if *parent != bone && depth < bones.len() => {
        Self::resolve(*parent, bones, index_of, transforms, depth + 1)
      }
      _ => Affine3d::IDENTITY,
    };
    let transform: Affine3d = parent.compose(&local);

    transforms[bone] = Some(transform);

    transform
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
