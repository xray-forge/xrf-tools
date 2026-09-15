use serde::Serialize;
use xrf_db::SkeletonPart;

/// The index a definition uses to say it names nothing, `BI_NONE` (`xrCore/Animation/Bone.hpp`).
const UNNAMED_INDEX: u16 = u16::MAX;

/// What a motion plays on.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchiveOmfTarget {
  // A cycle, played on one part of the partition. The name is absent for an index the partition does not declare.
  Part { index: u16, name: Option<String> },
  // An effect, played on one bone. Named when the partition happens to list the bone, which it does only for bones
  // some part drives.
  Bone { index: u16, name: Option<String> },
  // `BI_NONE`: the definition names neither, which 70% of vanilla's cycles do.
  Unnamed,
}

impl ArchiveOmfTarget {
  /// What a definition's `bone_or_part` addresses, resolved against the partition it was read beside.
  pub fn of(index: u16, is_fx: bool, parts: &[SkeletonPart]) -> Self {
    if index == UNNAMED_INDEX {
      return Self::Unnamed;
    }

    if is_fx {
      Self::Bone {
        index,
        name: Self::find_bone(index, parts),
      }
    } else {
      Self::Part {
        index,
        name: parts.get(usize::from(index)).map(|part| part.name.clone()),
      }
    }
  }

  /// The bone a global index names, looked up across every part because the partition is what holds the names.
  fn find_bone(index: u16, parts: &[SkeletonPart]) -> Option<String> {
    parts
      .iter()
      .flat_map(|part| &part.bones)
      .find(|(_, bone)| *bone == u32::from(index))
      .map(|(name, _)| name.clone())
  }
}

#[cfg(test)]
mod tests {
  use xrf_db::SkeletonPart;

  use super::ArchiveOmfTarget;

  fn partition() -> Vec<SkeletonPart> {
    vec![
      SkeletonPart {
        name: String::from("torso"),
        bones: vec![(String::from("bip01_spine"), 0), (String::from("bip01_head"), 1)],
      },
      SkeletonPart {
        name: String::from("legs"),
        bones: vec![(String::from("bip01_l_thigh"), 2)],
      },
    ]
  }

  #[test]
  fn a_cycle_names_the_part_it_plays_on() {
    assert_eq!(
      ArchiveOmfTarget::of(1, false, &partition()),
      ArchiveOmfTarget::Part {
        index: 1,
        name: Some(String::from("legs"))
      }
    );
  }

  #[test]
  fn an_effect_names_the_bone_it_plays_on() {
    assert_eq!(
      ArchiveOmfTarget::of(2, true, &partition()),
      ArchiveOmfTarget::Bone {
        index: 2,
        name: Some(String::from("bip01_l_thigh"))
      }
    );
  }

  #[test]
  fn an_index_the_partition_does_not_declare_keeps_its_number() {
    assert_eq!(
      ArchiveOmfTarget::of(7, false, &partition()),
      ArchiveOmfTarget::Part { index: 7, name: None }
    );
    assert_eq!(
      ArchiveOmfTarget::of(7, true, &partition()),
      ArchiveOmfTarget::Bone { index: 7, name: None }
    );
  }

  #[test]
  fn the_index_that_names_nothing_is_read_as_naming_nothing() {
    assert_eq!(
      ArchiveOmfTarget::of(u16::MAX, false, &partition()),
      ArchiveOmfTarget::Unnamed
    );
    assert_eq!(
      ArchiveOmfTarget::of(u16::MAX, true, &partition()),
      ArchiveOmfTarget::Unnamed
    );
  }
}
