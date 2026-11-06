use std::collections::HashMap;

use xrf_editable_object::EditableBone;
use xrf_math::Affine3d;

/// Which bone hangs under which, `CEditableObject::PrepareBones`: a bone's parent found by name, ignoring case as the
/// SDK lowercases both, and an order visiting every parent before its children.
pub(crate) struct SkeletonHierarchy {
  /// Each bone's parent by index, none for a root.
  pub(crate) parents: Vec<Option<usize>>,
  /// Every bone once, each after its parent.
  order: Vec<usize>,
}

impl SkeletonHierarchy {
  pub(crate) fn of(bones: &[EditableBone]) -> Self {
    let index_of: HashMap<String, usize> = bones
      .iter()
      .enumerate()
      .map(|(index, bone)| (bone.definition.name.to_ascii_lowercase(), index))
      .collect();
    let mut parents: Vec<Option<usize>> = bones
      .iter()
      .enumerate()
      .map(|(index, bone)| {
        index_of
          .get(&bone.definition.parent.to_ascii_lowercase())
          .copied()
          .filter(|parent| *parent != index)
      })
      .collect();
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); bones.len()];

    for (index, parent) in parents.iter().enumerate() {
      if let Some(parent) = parent {
        children[*parent].push(index);
      }
    }

    let mut order: Vec<usize> = Vec::with_capacity(bones.len());
    let mut is_visited: Vec<bool> = vec![false; bones.len()];
    let visit = |root: usize, order: &mut Vec<usize>, is_visited: &mut Vec<bool>| {
      let mut pending: Vec<usize> = vec![root];

      while let Some(bone) = pending.pop() {
        if !std::mem::replace(&mut is_visited[bone], true) {
          order.push(bone);
          pending.extend(children[bone].iter().rev());
        }
      }
    };

    for root in (0..bones.len()).filter(|index| parents[*index].is_none()) {
      visit(root, &mut order, &mut is_visited);
    }

    // A parent chain that loops reaches no root; its bones stand as roots, as no transform reaches them otherwise.
    for bone in 0..bones.len() {
      if !is_visited[bone] {
        parents[bone] = None;
        visit(bone, &mut order, &mut is_visited);
      }
    }

    Self { parents, order }
  }

  /// Every bone once, each after its parent.
  pub(crate) fn order(&self) -> impl Iterator<Item = usize> + '_ {
    self.order.iter().copied()
  }

  /// Each bone's transform in the object from each's own under its parent, `CalculateRest` and `CalculateAnim`.
  pub(crate) fn to_object(&self, local: &[Affine3d]) -> Vec<Affine3d> {
    let mut placed: Vec<Affine3d> = local.to_vec();

    for bone in &self.order {
      if let Some(parent) = self.parents[*bone] {
        placed[*bone] = placed[parent].compose(&local[*bone]);
      }
    }

    placed
  }

  /// Each bone's rest transform in the object, `CalculateRest`: its bind rotation (`setXYZi`) and offset under its
  /// parent's.
  pub(crate) fn rest_transforms(&self, bones: &[EditableBone]) -> Vec<Affine3d> {
    self.to_object(
      &bones
        .iter()
        .map(|bone| Affine3d::from_xyz_inverse(&bone.bind_pose.rotation).with_translation(bone.bind_pose.offset))
        .collect::<Vec<_>>(),
    )
  }
}
