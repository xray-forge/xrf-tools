use xrf_db::{OgfBone, OgfBoneIkData};

use crate::data::visual_description::VisualBone;
use crate::pack::visual_transform::BindTransform;

/// Converts a bone list into the renderer-facing skeleton, resolving the bind pose when the file carries one.
///
/// Each bone's model-space bind transform and its parent's index survive: the transform's translation is what draws a
/// skeleton, and the whole transform is what skinning inverts. Composition stays on this side of the wire, which is
/// where a sign or an operand order goes wrong invisibly, and transforms leave in renderer space so the skeleton lands
/// in the geometry rather than mirrored beside it.
///
/// A visual with no IK chunk still reports its bone names and parents, with no transforms: the hierarchy is worth
/// listing even when nothing can be drawn from it.
pub fn convert_bones(bones: &[OgfBone], ik_data: Option<&[OgfBoneIkData]>) -> Vec<VisualBone> {
  // The engine reads one IK record per bone, in bone order (`SkeletonCustom.cpp:297`), so a chunk of a different
  // length is not a chunk this pairing understands.
  let binds: Option<&[OgfBoneIkData]> = ik_data.filter(|it| it.len() == bones.len());
  let model: Vec<Option<BindTransform>> = match binds {
    Some(binds) => resolve_model_transforms(bones, binds),
    None => vec![None; bones.len()],
  };

  bones
    .iter()
    .enumerate()
    .map(|(index, bone)| VisualBone {
      name: bone.name.clone(),
      parent: bone.parent.clone(),
      parent_index: find_parent(bones, &bone.parent).map(|it| it as u32),
      bind_transform: model[index].as_ref().map(BindTransform::to_renderer_space),
    })
    .collect()
}

/// Every bone's model-space bind transform, or `None` for a bone whose parent chain does not reach a root.
///
/// Model space, not world space: each transform is composed up the parent chain from identity at the root bone, and
/// nothing here knows where the object sits in a level. World space would need that placement, which OGF bone data
/// does not carry.
///
/// Resolved by repeated passes rather than by recursion: the format does not promise parents precede children, and a
/// chain that refers to itself must terminate rather than overflow the stack. Each pass places every bone whose parent
/// is already placed, so a pass that places nothing means the rest is unreachable.
fn resolve_model_transforms(bones: &[OgfBone], binds: &[OgfBoneIkData]) -> Vec<Option<BindTransform>> {
  let local: Vec<BindTransform> = binds
    .iter()
    .map(|it| BindTransform::from_bind(&it.bind_rotation, &it.bind_position))
    .collect();
  let parents: Vec<Option<usize>> = bones.iter().map(|it| find_parent(bones, &it.parent)).collect();

  let mut model: Vec<Option<BindTransform>> = vec![None; bones.len()];

  loop {
    let mut placed: usize = 0;

    for index in 0..bones.len() {
      if model[index].is_some() {
        continue;
      }

      let resolved: Option<BindTransform> = match parents[index] {
        // A root bone's parent is the identity, which is what the engine walks down from.
        None => Some(local[index].then(&BindTransform::identity())),
        Some(parent) => model[parent].as_ref().map(|it| local[index].then(it)),
      };

      if let Some(resolved) = resolved {
        model[index] = Some(resolved);
        placed += 1;
      }
    }

    if placed == 0 {
      return model;
    }
  }
}

/// The index of the bone a name refers to, or `None` for a root or a name no bone carries.
fn find_parent(bones: &[OgfBone], parent: &str) -> Option<usize> {
  if parent.is_empty() {
    return None;
  }

  bones.iter().position(|it| it.name == parent)
}
