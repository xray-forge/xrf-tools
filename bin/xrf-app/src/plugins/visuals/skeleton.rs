use xrf_db::{OgfBone, OgfBoneIkData, OgfFile, SkeletonMotion, SkeletonMotionDefinition, SkeletonPart};

/// What posing a visual needs from its file, kept beside the selection so a motion costs no re-read.
pub struct SelectedSkeleton {
  pub bones: Vec<OgfBone>,
  pub binds: Vec<OgfBoneIkData>,
  /// Partitions of the motions this file embeds, empty when it animates only from referenced omf files.
  pub embedded_parts: Vec<SkeletonPart>,
  /// Motions the file carries itself, each definition with the key payload at its own ordinal.
  pub embedded_motions: Vec<(SkeletonMotionDefinition, SkeletonMotion)>,
}

impl SelectedSkeleton {
  /// What a visual offers for posing, or nothing when it carries no bind pose to pose from.
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
