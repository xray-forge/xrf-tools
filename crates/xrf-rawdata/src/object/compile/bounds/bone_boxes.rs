use xrf_editable_object::EditableBone;
use xrf_math::glam::{DVec3, Vec3};
use xrf_math::{Affine3d, Obb, Vector3d};

use crate::object::compile::bounds::flush_box::FlushBox;
use crate::object::compile::skeleton::skeleton_hierarchy::SkeletonHierarchy;
use crate::object::compile::skeleton::skeleton_split::SkeletonSplit;

/// The box stored beside each bone's name, in the bone's rest space: the engine moves it with the bone for culling
/// (`CKinematics::CalculateBones`), decal placement (`CKinematics::AddWallmark`) and AI obstacles; bullet hits test the
/// IK shapes instead.
///
/// Each box is the [`FlushBox`] around every vertex linked to the bone with any weight, where the export takes only a
/// vertex's lowest-numbered bone: through vanilla's motions that leaves 1.7% of vertices over a centimetre outside their
/// decal boxes, and this 0.03%, with boxes 11% smaller in sum. A bone no vertex is linked to gets `Fobb::invalidate`'s
/// empty box.
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
