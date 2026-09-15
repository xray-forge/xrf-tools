use serde::Serialize;
use xrf_db::OmfFile;

use crate::plugins::archives::describe::omf::archive_omf_motion::ArchiveOmfMotion;
use crate::plugins::archives::describe::omf::archive_omf_motion_flag::ArchiveOmfMotionFlag;

/// The version that appended per-motion marks, which is what a bank below it cannot carry.
const MARKS_VERSION: u16 = 4;

/// What a bank holds, taken over the whole of it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveOmfBank {
  pub version: u16,
  /// Whether the version carries motion marks at all, which version 3 does not.
  pub carries_marks: bool,
  pub motions: usize,
  /// Motions flagged `esmFX`, which play on a bone rather than on a part of the partition.
  pub effects: usize,
  pub bones: usize,
  pub frames: u32,
  /// Seconds every motion together spans at the format's fixed sample rate, before playback speed applies.
  pub duration_seconds: f32,
  /// Motions whose payload still carries a name that is not theirs.
  pub diverging_labels: usize,
  /// Motions carrying at least one mark.
  pub marked_motions: usize,
  /// Motions whose declared falloff the engine replaces on load.
  pub replaced_falloffs: usize,
}

impl ArchiveOmfBank {
  /// The totals of a bank, taken off the motions already described so the two cannot disagree.
  pub fn of(file: &OmfFile, motions: &[ArchiveOmfMotion]) -> Self {
    let frames: u32 = motions.iter().map(|motion| motion.frames).sum();

    Self {
      version: file.parameters.version,
      carries_marks: file.parameters.version >= MARKS_VERSION,
      motions: motions.len(),
      // Off the flag word rather than off the described target, which reads `BI_NONE` as naming nothing and so
      // cannot tell the twelve vanilla effects that target no bone from a cycle that targets no part.
      effects: file
        .parameters
        .motions
        .iter()
        .filter(|definition| ArchiveOmfMotionFlag::Fx.is_set_in(definition.flags))
        .count(),
      bones: file.get_bones_count(),
      frames,
      duration_seconds: ArchiveOmfMotion::to_duration_seconds(frames),
      diverging_labels: motions.iter().filter(|motion| motion.has_diverging_label).count(),
      marked_motions: motions.iter().filter(|motion| !motion.marks.is_empty()).count(),
      replaced_falloffs: motions.iter().filter(|motion| motion.blend.is_falloff_replaced).count(),
    }
  }
}
