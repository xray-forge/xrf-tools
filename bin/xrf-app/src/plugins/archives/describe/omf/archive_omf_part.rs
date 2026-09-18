use serde::Serialize;
use xrf_skeleton::{SkeletonMotionDefinition, SkeletonPart};

use crate::plugins::archives::describe::omf::archive_omf_motion_flag::ArchiveOmfMotionFlag;

/// One part of a bank's partition, and what plays on it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveOmfPart {
  pub name: String,
  /// Bones the part drives, in the order it lists them.
  pub bones: Vec<String>,
  /// Cycles this bank routes to the part, which is what makes it more than a name.
  pub cycles: usize,
}

impl ArchiveOmfPart {
  /// Every part of a partition, each counting the cycles that name it.
  pub fn of_all(parts: &[SkeletonPart], motions: &[SkeletonMotionDefinition]) -> Vec<Self> {
    parts
      .iter()
      .enumerate()
      .map(|(index, part)| Self {
        name: part.name.clone(),
        bones: part.bones.iter().map(|(name, _)| name.clone()).collect(),
        cycles: motions
          .iter()
          .filter(|definition| {
            // An effect's index is a bone rather than a part, so counting it here would credit the part sharing its
            // number with animations that never play on it.
            !ArchiveOmfMotionFlag::Fx.is_set_in(definition.flags) && usize::from(definition.bone_or_part) == index
          })
          .count(),
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use xrf_skeleton::{SkeletonMotionDefinition, SkeletonPart};

  use super::ArchiveOmfPart;

  fn definition(flags: u32, bone_or_part: u16) -> SkeletonMotionDefinition {
    SkeletonMotionDefinition {
      name: String::from("motion"),
      flags,
      bone_or_part,
      motion: 0,
      speed: 1.0,
      power: 1.0,
      accrue: 2.0,
      falloff: 2.0,
      marks: Vec::new(),
    }
  }

  #[test]
  fn a_part_counts_the_cycles_that_name_it_and_not_the_effects() {
    let parts: Vec<SkeletonPart> = vec![
      SkeletonPart {
        name: String::from("torso"),
        bones: vec![(String::from("bip01_spine"), 0)],
      },
      SkeletonPart {
        name: String::from("legs"),
        bones: Vec::new(),
      },
    ];

    let described: Vec<ArchiveOmfPart> = ArchiveOmfPart::of_all(
      &parts,
      &[
        definition(0, 0),
        definition(0, 0),
        definition(0, 1),
        // An effect on bone 0, which is not a cycle on part 0.
        definition(1, 0),
        // The index that names nothing.
        definition(0, u16::MAX),
      ],
    );

    assert_eq!(described[0].name, "torso");
    assert_eq!(described[0].bones, vec![String::from("bip01_spine")]);
    assert_eq!(described[0].cycles, 2);
    assert_eq!(described[1].cycles, 1);
  }
}
