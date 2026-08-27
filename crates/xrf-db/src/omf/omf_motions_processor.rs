use std::collections::{HashMap, HashSet};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::assert_equal;

use crate::omf::omf_file::OmfFile;

/// Editing operations over the motion set of an omf file.
pub struct OmfMotionsProcessor {}

impl OmfMotionsProcessor {
  /// Bit that makes a motion play once and stop rather than loop.
  pub const FLAG_PLAY_ONCE: u32 = 0b10;

  /// Keep only motions whose definition name is accepted by the predicate.
  ///
  /// Definitions and payloads are dropped together so the surviving ones still pair by ordinal.
  ///
  /// `motion` ids are renumbered to the new ordinals only when the source file already carried ordinal ids. No engine
  /// code reads the field, and third-party banks exist whose ids are neither ordinals nor anything else we can
  /// account for, so those are carried through untouched rather than replaced with a meaning we invented.
  ///
  /// Returns the count of removed motions.
  pub fn retain_motions(file: &mut OmfFile, predicate: impl Fn(&str) -> bool) -> XrfResult<usize> {
    Self::assert_motions_are_paired(file, "filtering")?;

    let renumber: bool = Self::has_ordinal_motion_ids(file);
    let retained: Vec<bool> = file.parameters.motions.iter().map(|it| predicate(&it.name)).collect();

    let removed_count: usize = retained.iter().filter(|it| !**it).count();
    let mut retained_iter = retained.iter();

    file
      .parameters
      .motions
      .retain(|_| *retained_iter.next().expect("Retained flag for each motion definition"));

    let mut retained_iter = retained.iter();

    file
      .motions
      .motions
      .retain(|_| *retained_iter.next().expect("Retained flag for each motion"));

    if renumber {
      for (ordinal, definition) in file.parameters.motions.iter_mut().enumerate() {
        definition.motion = u16::try_from(ordinal)
          .map_err(|_| XrfError::new_invalid_error("Motions count exceeds the supported range after filtering"))?;
      }
    }

    Ok(removed_count)
  }

  /// Rename motions using the provided old name to new name map.
  ///
  /// The definition name is the lookup key the engine resolves against, so it is what a rename changes. A payload's
  /// label follows only when it still matched the name being replaced: a `_DEBUG` engine build asserts the two agree,
  /// so a bank that was consistent stays consistent, while a stale or non-text label is preserved rather than
  /// overwritten with a value we cannot vouch for.
  ///
  /// Returns the count of renamed motions.
  pub fn rename_motions(file: &mut OmfFile, renames: &HashMap<String, String>) -> XrfResult<usize> {
    Self::assert_motions_are_paired(file, "renaming")?;

    let mut renamed_count: usize = 0;

    for (definition, motion) in file.parameters.motions.iter_mut().zip(file.motions.motions.iter_mut()) {
      let Some(renamed) = renames.get(&definition.name) else {
        continue;
      };

      if motion.has_label_matching(&definition.name) {
        motion.label.clone_from(renamed);
      }

      definition.name.clone_from(renamed);
      renamed_count += 1;
    }

    Self::assert_motion_names_are_unique(file)?;

    Ok(renamed_count)
  }

  /// Copy an existing motion under a new name, optionally forcing it to play once.
  ///
  /// Both the definition and the keyframe payload are duplicated rather than aliased, because the
  /// rest of this processor treats the two lists as ordinal pairs and a definition without a
  /// payload would break every later filter or rename.
  pub fn duplicate_motion(file: &mut OmfFile, from: &str, to: &str, play_once: bool) -> XrfResult {
    Self::assert_motions_are_paired(file, "duplicating")?;

    let index: usize = file
      .parameters
      .motions
      .iter()
      .position(|it| it.name == from)
      .ok_or_else(|| XrfError::new_not_found_error(format!("Motion '{from}' was not found in the omf file")))?;

    let renumber: bool = Self::has_ordinal_motion_ids(file);

    let mut definition = file.parameters.motions[index].clone();
    let mut motion = file.motions.motions[index].clone();

    // The label follows the new name only where it still described the motion it was copied from.
    if motion.has_label_matching(&definition.name) {
      motion.label = String::from(to);
    }

    definition.name = String::from(to);

    if play_once {
      definition.flags |= Self::FLAG_PLAY_ONCE;
    }

    if renumber {
      definition.motion = u16::try_from(file.parameters.motions.len())
        .map_err(|_| XrfError::new_invalid_error("Motions count exceeds the supported range after duplication"))?;
    }

    file.parameters.motions.push(definition);
    file.motions.motions.push(motion);

    Self::assert_motion_names_are_unique(file)?;

    Ok(())
  }

  /// Whether every definition's `motion` id already equals its own ordinal.
  ///
  /// True for vanilla banks and for everything the workspace corpus carries outside a handful of third-party files.
  /// It is the only evidence available that a file's ids mean ordinals at all, so editing preserves them when it is
  /// false.
  fn has_ordinal_motion_ids(file: &OmfFile) -> bool {
    file
      .parameters
      .motions
      .iter()
      .enumerate()
      .all(|(ordinal, definition)| usize::from(definition.motion) == ordinal)
  }

  /// Guard that definitions and payloads can be treated as ordinal pairs.
  fn assert_motions_are_paired(file: &OmfFile, operation: &str) -> XrfResult {
    assert_equal(
      file.parameters.motions.len(),
      file.motions.motions.len(),
      &format!("Expect matching motions and motion definitions count before {operation}"),
    )
  }

  /// Guard that no two motions share a name, which would make one of them unreachable.
  fn assert_motion_names_are_unique(file: &OmfFile) -> XrfResult {
    let mut seen: HashSet<&str> = HashSet::new();

    for definition in &file.parameters.motions {
      if !seen.insert(&definition.name) {
        return Err(XrfError::new_invalid_error(format!(
          "Motion name '{}' is duplicated, motion names must be unique within a file",
          definition.name
        )));
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use xrf_error::XrfResult;

  use crate::data::ogf::ogf_motion::OgfMotion;
  use crate::data::ogf::ogf_motion_definition::OgfMotionDefinition;
  use crate::data::ogf::ogf_part::OgfPart;
  use crate::omf::chunks::omf_motions_chunk::OmfMotionsChunk;
  use crate::omf::chunks::omf_parameters_chunk::OmfParametersChunk;
  use crate::omf::omf_file::OmfFile;
  use crate::omf::omf_motions_processor::OmfMotionsProcessor;

  /// Build a file whose motions are named after the provided list.
  fn new_named_mock(names: &[&str]) -> OmfFile {
    OmfFile {
      parameters: OmfParametersChunk {
        version: 4,
        parts: vec![OgfPart {
          name: String::from("default"),
          bones: vec![(String::from("bip01"), 0)],
        }],
        motions: names
          .iter()
          .enumerate()
          .map(|(ordinal, name)| {
            let mut definition: OgfMotionDefinition = OgfMotionDefinition::new_mock(Vec::new());

            definition.name = String::from(*name);
            definition.motion = ordinal as u16;
            definition
          })
          .collect(),
      },
      motions: OmfMotionsChunk {
        motions: names
          .iter()
          .map(|name| OgfMotion {
            label: String::from(*name),
            count: 1,
            flags: 0,
            remaining: vec![1, 2, 3],
          })
          .collect(),
      },
    }
  }

  /// Windows-1251 decoding of the bytes `92 3C D7`, the start of one real scrambled payload label.
  const SCRAMBLED_LABEL: &str = "\u{2019}<\u{0427}";

  /// Build a file that reproduces a third-party bank: payload labels that are not the motion names,
  /// and `motion` ids that are not ordinals. Both are preserved rather than normalised.
  fn new_divergent_mock(names: &[&str]) -> OmfFile {
    let mut file: OmfFile = new_named_mock(names);

    for (ordinal, (definition, motion)) in file
      .parameters
      .motions
      .iter_mut()
      .zip(file.motions.motions.iter_mut())
      .enumerate()
    {
      definition.motion = 50_000 + ordinal as u16;
      motion.label = String::from(SCRAMBLED_LABEL);
    }

    file
  }

  #[test]
  fn test_retain_motions_filters_both_lists_and_reindexes() -> XrfResult {
    let mut file: OmfFile = new_named_mock(&["ak_74_draw", "aek_draw", "ak_74_idle", "akm_idle"]);

    assert_eq!(
      OmfMotionsProcessor::retain_motions(&mut file, |name| name.starts_with("ak_74_"))?,
      2
    );

    assert_eq!(file.get_motion_names(), vec!["ak_74_draw", "ak_74_idle"]);
    assert_eq!(
      file
        .motions
        .motions
        .iter()
        .map(|it| it.label.as_str())
        .collect::<Vec<_>>(),
      vec!["ak_74_draw", "ak_74_idle"],
      "Expect motion payloads to be filtered alongside definitions"
    );
    assert_eq!(
      file.parameters.motions.iter().map(|it| it.motion).collect::<Vec<_>>(),
      vec![0, 1],
      "Expect surviving definitions to be renumbered to their new ordinal"
    );

    Ok(())
  }

  #[test]
  fn test_retain_motions_keeps_payload_paired_with_definition() -> XrfResult {
    let mut file: OmfFile = new_named_mock(&["first", "second", "third"]);

    file.motions.motions[1].remaining = vec![42];

    OmfMotionsProcessor::retain_motions(&mut file, |name| name == "second")?;

    assert_eq!(
      file.motions.motions.first().expect("Retained motion").remaining,
      vec![42],
      "Expect the payload of the retained motion to survive, not the payload at index zero"
    );

    Ok(())
  }

  #[test]
  fn test_rename_motions_updates_definitions_and_payloads() -> XrfResult {
    let mut file: OmfFile = new_named_mock(&["ak_74_draw", "ak_74_idle_move"]);

    let renames: HashMap<String, String> = HashMap::from([
      (String::from("ak_74_draw"), String::from("ak74_draw")),
      (String::from("ak_74_idle_move"), String::from("ak74_idle_moving")),
    ]);

    assert_eq!(OmfMotionsProcessor::rename_motions(&mut file, &renames)?, 2);

    assert_eq!(file.get_motion_names(), vec!["ak74_draw", "ak74_idle_moving"]);
    assert_eq!(
      file
        .motions
        .motions
        .iter()
        .map(|it| it.label.as_str())
        .collect::<Vec<_>>(),
      vec!["ak74_draw", "ak74_idle_moving"],
      "Expect a payload label that matched its motion name to follow the rename, which _DEBUG playback asserts"
    );

    Ok(())
  }

  #[test]
  fn test_rename_motions_leaves_unmapped_names() -> XrfResult {
    let mut file: OmfFile = new_named_mock(&["ak_74_draw", "ak_74_idle"]);

    let renames: HashMap<String, String> = HashMap::from([(String::from("ak_74_draw"), String::from("ak74_draw"))]);

    assert_eq!(OmfMotionsProcessor::rename_motions(&mut file, &renames)?, 1);
    assert_eq!(file.get_motion_names(), vec!["ak74_draw", "ak_74_idle"]);

    Ok(())
  }

  #[test]
  fn test_rename_motions_rejects_duplicates() {
    let mut file: OmfFile = new_named_mock(&["first", "second"]);

    let renames: HashMap<String, String> = HashMap::from([(String::from("first"), String::from("second"))]);

    assert!(
      OmfMotionsProcessor::rename_motions(&mut file, &renames).is_err(),
      "Expect rename producing duplicate names to be rejected"
    );
  }

  #[test]
  fn test_duplicate_motion_copies_both_lists_and_points_at_its_own_payload() -> XrfResult {
    let mut file: OmfFile = new_named_mock(&["pm_idle", "pm_shoot"]);

    OmfMotionsProcessor::duplicate_motion(&mut file, "pm_idle", "pm_idle_bore", false)?;

    assert_eq!(file.parameters.motions.len(), 3);
    assert_eq!(file.motions.motions.len(), 3);
    assert_eq!(file.parameters.motions[2].name, "pm_idle_bore");
    assert_eq!(file.motions.motions[2].label, "pm_idle_bore");
    // The copy must address its own new slot, not the ordinal it was cloned from.
    assert_eq!(file.parameters.motions[2].motion, 2);

    Ok(())
  }

  #[test]
  fn test_duplicate_motion_can_clear_looping() -> XrfResult {
    let mut file: OmfFile = new_named_mock(&["pm_idle"]);

    // Start from a looping motion, which is what a real idle is.
    file.parameters.motions[0].flags = 0;

    OmfMotionsProcessor::duplicate_motion(&mut file, "pm_idle", "pm_idle_bore", true)?;

    // Without this the engine never leaves the bore state, since it exits only on animation end.
    assert_ne!(
      file.parameters.motions[1].flags & OmfMotionsProcessor::FLAG_PLAY_ONCE,
      0
    );
    // The source keeps looping.
    assert_eq!(
      file.parameters.motions[0].flags & OmfMotionsProcessor::FLAG_PLAY_ONCE,
      0
    );

    Ok(())
  }

  #[test]
  fn test_duplicate_motion_rejects_an_unknown_source() {
    let mut file: OmfFile = new_named_mock(&["pm_idle"]);

    assert!(OmfMotionsProcessor::duplicate_motion(&mut file, "pm_missing", "pm_idle_bore", true).is_err());
  }

  #[test]
  fn test_duplicate_motion_rejects_a_name_already_present() {
    let mut file: OmfFile = new_named_mock(&["pm_idle", "pm_shoot"]);

    assert!(OmfMotionsProcessor::duplicate_motion(&mut file, "pm_idle", "pm_shoot", true).is_err());
  }
  #[test]
  fn test_retain_motions_preserves_non_ordinal_motion_ids() -> XrfResult {
    let mut file: OmfFile = new_divergent_mock(&["first", "second", "third"]);

    OmfMotionsProcessor::retain_motions(&mut file, |name| name != "second")?;

    assert_eq!(
      file.parameters.motions.iter().map(|it| it.motion).collect::<Vec<_>>(),
      vec![50_000, 50_002],
      "Expect ids that were never ordinals to survive filtering untouched"
    );

    Ok(())
  }

  #[test]
  fn test_rename_motions_preserves_a_diverging_payload_label() -> XrfResult {
    let mut file: OmfFile = new_divergent_mock(&["first", "second"]);

    let renames: HashMap<String, String> = HashMap::from([(String::from("first"), String::from("primary"))]);

    assert_eq!(OmfMotionsProcessor::rename_motions(&mut file, &renames)?, 1);

    assert_eq!(file.get_motion_names(), vec!["primary", "second"]);
    assert_eq!(
      file.motions.motions[0].label, SCRAMBLED_LABEL,
      "Expect a label that never named its motion to stay exactly as the file stored it"
    );

    Ok(())
  }

  #[test]
  fn test_rename_motions_leaves_labels_of_motions_it_did_not_rename() -> XrfResult {
    let mut file: OmfFile = new_named_mock(&["first", "second"]);

    // A stale label on a motion nobody asked to rename: an unrelated rename must not touch it.
    file.motions.motions[1].label = String::from("second_old");

    let renames: HashMap<String, String> = HashMap::from([(String::from("first"), String::from("primary"))]);

    OmfMotionsProcessor::rename_motions(&mut file, &renames)?;

    assert_eq!(file.motions.motions[0].label, "primary");
    assert_eq!(file.motions.motions[1].label, "second_old");

    Ok(())
  }

  #[test]
  fn test_duplicate_motion_preserves_a_diverging_label_and_motion_id() -> XrfResult {
    let mut file: OmfFile = new_divergent_mock(&["pm_idle", "pm_shoot"]);

    OmfMotionsProcessor::duplicate_motion(&mut file, "pm_idle", "pm_idle_bore", false)?;

    assert_eq!(file.parameters.motions[2].name, "pm_idle_bore");
    assert_eq!(
      file.motions.motions[2].label, SCRAMBLED_LABEL,
      "Expect the copy to carry the label it was cloned from, not a name we invented"
    );
    assert_eq!(
      file.parameters.motions[2].motion, 50_000,
      "Expect the cloned id to be preserved when the file never used ordinal ids"
    );

    Ok(())
  }
}
