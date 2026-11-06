use xrf_editable_object::EditableBone;

/// The breakable part each bone belongs to, as `BuildGroups` numbers them for the skeleton export.
///
/// Walking down from the root, a breakable bone (`SJointIKData::flBreakable`) opens a new part, numbered in the order
/// the walk reaches it, and every other bone stays in its parent's. An object with no breakable bone is one part.
pub(crate) struct BreakableParts;

impl BreakableParts {
  /// `SJointIKData::flBreakable`.
  const BREAKABLE_FLAG: u32 = 1 << 0;

  /// Each bone's part, by bone index, all 0 where no bone is breakable.
  pub(crate) fn of(bones: &[EditableBone]) -> Vec<u16> {
    let mut parts: Vec<u16> = vec![0; bones.len()];
    let is_breakable = |bone: &EditableBone| bone.joint_flags.unwrap_or(0) & Self::BREAKABLE_FLAG != 0;

    // The editor keeps the last root it finds.
    let Some(root) = bones.iter().rposition(|bone| bone.definition.parent.is_empty()) else {
      return parts;
    };

    if !bones.iter().any(is_breakable) {
      return parts;
    }

    let mut last: u16 = 0;
    let mut pending: Vec<(usize, u16)> = vec![(root, 0)];

    // Depth first, children in bone order, as the editor's recursion visits them.
    while let Some((bone, inherited)) = pending.pop() {
      let part: u16 = if is_breakable(&bones[bone]) {
        last += 1;
        last
      } else {
        inherited
      };

      parts[bone] = part;

      let name: &str = &bones[bone].definition.name;

      for child in (0..bones.len()).rev() {
        if bones[child].definition.parent == name {
          pending.push((child, part));
        }
      }
    }

    parts
  }
}
