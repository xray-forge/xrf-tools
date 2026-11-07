use glam::{DVec3, Vec3};
use xrf_editable_object::EditableBone;
use xrf_math::{Affine3d, Obb, Vector3d};

use crate::object::compile::bounds::flush_box::FlushBox;
use crate::object::compile::skeleton::skeleton_hierarchy::SkeletonHierarchy;
use crate::object::compile::skeleton::skeleton_split::SkeletonSplit;

/// The box stored beside each bone's name, in the bone's rest space: the engine moves it with the bone to bound the
/// animated model for culling (`CKinematics::CalculateBones`), to gate a decal's per-triangle placement on the bone
/// (`CKinematics::AddWallmark`) and as an AI obstacle of a physics object. Bullet hits test the IK data's shapes
/// instead, so no damage depends on it.
///
/// Each box is the [`FlushBox`] around every vertex linked to the bone with any weight, as Godot boxes its bones. The
/// export gathers a vertex into the box of its lowest-numbered bone alone, so a vertex blended toward a child leaves its
/// box as the child turns: posed through vanilla's motions, 1.7% of vanilla's vertices end more than a centimetre
/// outside every box gating their triangles' decals, and the animated bounds lose one by a centimetre in 0.9% of frames. Every linked vertex brings
/// that to 0.03% and none, with boxes still 11% smaller in sum. A bone no vertex is linked to gets the empty box
/// `Fobb::invalidate` leaves.
pub(crate) struct BoneBoxes;

impl BoneBoxes {
  /// Each bone's box, in bone order.
  pub(crate) fn of(bones: &[EditableBone], splits: &[SkeletonSplit]) -> Vec<Obb> {
    let into_bone: Vec<Affine3d> = SkeletonHierarchy::of(bones)
      .rest_transforms(bones)
      .iter()
      .map(Affine3d::inverse_rigid)
      .collect();
    let mut points: Vec<Vec<DVec3>> = vec![Vec::new(); bones.len()];

    for vertex in splits.iter().flat_map(|split| &split.vertices) {
      // A weightless bone is the export's padding, which repeats a bone the vertex follows already.
      for bone in vertex.bones.iter().filter(|it| it.weight > 0.0) {
        let index: usize = usize::from(bone.bone);

        if let Some(transform) = into_bone.get(index) {
          points[index].push(Vec3::from(transform.transform_point(&vertex.position)).as_dvec3());
        }
      }
    }

    points
      .iter()
      .map(|points| FlushBox::around(points).map_or_else(Self::empty, |fitted| Obb::from(&fitted)))
      .collect()
  }

  /// `Fobb::invalidate`'s box: no extent, at the origin, along the bone's axes.
  fn empty() -> Obb {
    Obb {
      rotation: [Vector3d::X, Vector3d::Y, Vector3d::Z],
      center: Vector3d::ZERO,
      half_extents: Vector3d::ZERO,
    }
  }
}
