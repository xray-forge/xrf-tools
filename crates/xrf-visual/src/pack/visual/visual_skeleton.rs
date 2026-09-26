use xrf_ogf::{OgfBone, OgfBoneIkData};

use crate::data::visual::skeleton::visual_bone::VisualBone;
use crate::pack::visual::visual_transform::BindTransform;

/// Converts a bone list into the renderer-facing skeleton, resolving the bind pose when the file carries one.
pub fn convert_bones(bones: &[OgfBone], ik_data: Option<&[OgfBoneIkData]>) -> Vec<VisualBone> {
  // The engine reads one IK record per bone, in bone order (`SkeletonCustom.cpp:297`), so a chunk of a different
  // length is not a chunk this pairing understands.
  let binds: Option<&[OgfBoneIkData]> = ik_data.filter(|it| it.len() == bones.len());
  let parents: Vec<Option<usize>> = resolve_parents(bones);
  let model: Vec<Option<BindTransform>> = match binds {
    Some(binds) => {
      let locals: Vec<BindTransform> = binds
        .iter()
        .map(|it| BindTransform::from_bind(&it.bind_rotation, &it.bind_position))
        .collect();

      BindTransform::compose_chain(&locals, &parents)
    }
    None => vec![None; bones.len()],
  };

  bones
    .iter()
    .enumerate()
    .map(|(index, bone)| VisualBone {
      name: bone.name.clone(),
      parent: bone.parent.clone(),
      parent_index: parents[index].map(|it| it as u32),
      bind_transform: model[index].as_ref().map(BindTransform::to_renderer_space),
    })
    .collect()
}

/// Each bone's parent, by index, or `None` for a root or a parent name no bone carries.
pub(crate) fn resolve_parents(bones: &[OgfBone]) -> Vec<Option<usize>> {
  bones.iter().map(|it| find_parent(bones, &it.parent)).collect()
}

/// The index of the bone a name refers to, or `None` for a root or a name no bone carries.
fn find_parent(bones: &[OgfBone], parent: &str) -> Option<usize> {
  if parent.is_empty() {
    return None;
  }

  bones.iter().position(|it| it.name == parent)
}
