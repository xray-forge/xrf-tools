use xrf_db::{OgfBone, OgfBoneIkData, OgfFile, SkeletonMotion, SkeletonMotionDefinition, SkeletonPart};

/// What posing a visual needs from its file, kept beside the selection so a motion costs no re-read.
///
/// Only the bones, their bind records, and the partitions of any motions the file embeds - a few kilobytes for a
/// skeleton of fifty bones, against megabytes for the file it came out of. Parking the whole `OgfFile` would hold the
/// vertex and index chunks for as long as a model stays open, and posing never looks at them.
pub struct SelectedSkeleton {
  pub bones: Vec<OgfBone>,
  pub binds: Vec<OgfBoneIkData>,
  /// Partitions of the motions this file embeds, empty when it animates only from referenced omf files.
  pub embedded_parts: Vec<SkeletonPart>,
  /// Motions the file carries itself, each definition with the key payload at its own ordinal.
  ///
  /// Held rather than re-read for the same reason the packed geometry is: the file has already been read once, and a
  /// self-animated visual is a prop or a weapon rather than a character, so its key payload is small. The pair is kept
  /// whole because a payload's own label is not the motion's name - only the definition names it.
  pub embedded_motions: Vec<(SkeletonMotionDefinition, SkeletonMotion)>,
}

impl SelectedSkeleton {
  /// What a visual offers for posing, or nothing when it carries no bind pose to pose from.
  ///
  /// Both chunks are required: bones without bind records give a hierarchy with no rest position, which is a tree to
  /// list rather than a skeleton to animate.
  pub fn of(file: &OgfFile) -> Option<Self> {
    let bones: &Vec<OgfBone> = &file.bones.as_ref()?.bones;
    let binds: &Vec<OgfBoneIkData> = &file.ik_data.as_ref()?.bones;

    if bones.len() != binds.len() {
      return None;
    }

    Some(Self {
      bones: bones.clone(),
      binds: binds.clone(),
      embedded_motions: file
        .get_motions()
        .map(|(definition, motion)| (definition.clone(), motion.clone()))
        .collect(),
      embedded_parts: file
        .motion_parameters
        .as_ref()
        .map(|it| it.parts.clone())
        .unwrap_or_default(),
    })
  }
}
