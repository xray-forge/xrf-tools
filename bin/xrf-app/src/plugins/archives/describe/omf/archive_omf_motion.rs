use serde::Serialize;
use xrf_db::{SAMPLE_FPS, SkeletonMotion, SkeletonMotionDefinition, SkeletonPart};

use crate::plugins::archives::describe::omf::archive_omf_blend::ArchiveOmfBlend;
use crate::plugins::archives::describe::omf::archive_omf_mark::ArchiveOmfMark;
use crate::plugins::archives::describe::omf::archive_omf_motion_flag::ArchiveOmfMotionFlag;
use crate::plugins::archives::describe::omf::archive_omf_quantized::ArchiveOmfQuantized;
use crate::plugins::archives::describe::omf::archive_omf_target::ArchiveOmfTarget;

/// One motion of a bank: what it is called, how long it is, and how the engine plays it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveOmfMotion {
  /// The name the engine resolves the motion by, which is the definition's and never the payload's label.
  pub name: String,
  pub frames: u32,
  /// Seconds the frames span at the format's fixed 30 fps, before playback speed applies.
  pub duration_seconds: f32,
  /// Seconds playing it actually takes, or `None` when the engine reads a speed of zero and the division has no answer.
  pub playback_seconds: Option<f32>,
  pub speed: ArchiveOmfQuantized,
  pub power: ArchiveOmfQuantized,
  pub blend: ArchiveOmfBlend,
  pub target: ArchiveOmfTarget,
  /// The named bits the definition's word carries, in bit order; only the set ones.
  pub flags: Vec<String>,
  /// Bits of the word no name here claims.
  pub unnamed_flags: u32,
  pub marks: Vec<ArchiveOmfMark>,
  /// Whether the payload still carries the name of the motion it holds.
  pub has_diverging_label: bool,
}

impl ArchiveOmfMotion {
  /// One motion, read against the partition of the bank it belongs to.
  pub fn of(definition: &SkeletonMotionDefinition, motion: &SkeletonMotion, parts: &[SkeletonPart]) -> Self {
    let is_fx: bool = ArchiveOmfMotionFlag::Fx.is_set_in(definition.flags);
    let speed: ArchiveOmfQuantized = ArchiveOmfQuantized::of(definition.speed);
    let duration_seconds: f32 = motion.get_duration_seconds();

    Self {
      name: definition.name.clone(),
      frames: motion.count,
      duration_seconds,
      // `CKinematicsAnimated::LL_MotionTime` divides the span by the speed it reads, so a bank declaring zero has no
      // playback time rather than an infinite one.
      playback_seconds: (speed.value > 0.0).then(|| duration_seconds / speed.value),
      speed,
      power: ArchiveOmfQuantized::of(definition.power),
      blend: ArchiveOmfBlend::of(definition.accrue, definition.falloff, is_fx),
      target: ArchiveOmfTarget::of(definition.bone_or_part, is_fx, parts),
      flags: ArchiveOmfMotionFlag::set_in(definition.flags),
      unnamed_flags: ArchiveOmfMotionFlag::unnamed_in(definition.flags),
      marks: definition.marks.iter().map(ArchiveOmfMark::of).collect(),
      has_diverging_label: !motion.has_label_matching(&definition.name),
    }
  }

  /// The seconds a motion's frames span, for a caller totalling a bank without joining its two lists again.
  pub fn to_duration_seconds(frames: u32) -> f32 {
    frames as f32 / SAMPLE_FPS
  }
}

#[cfg(test)]
mod tests {
  use xrf_db::{SkeletonMotion, SkeletonMotionDefinition, SkeletonMotionMark};

  use super::ArchiveOmfMotion;
  use crate::plugins::archives::describe::omf::archive_omf_mark::ArchiveOmfMarkInterval;
  use crate::plugins::archives::describe::omf::archive_omf_target::ArchiveOmfTarget;

  fn definition(flags: u32, speed: f32) -> SkeletonMotionDefinition {
    SkeletonMotionDefinition {
      name: String::from("idle"),
      flags,
      bone_or_part: u16::MAX,
      motion: 0,
      speed,
      power: 1.0,
      accrue: 2.0,
      falloff: 2.0,
      marks: Vec::new(),
    }
  }

  fn payload(label: &str, count: u32) -> SkeletonMotion {
    SkeletonMotion {
      label: String::from(label),
      count,
      flags: 0,
      remaining: Vec::new(),
    }
  }

  #[test]
  fn a_motion_takes_its_frames_from_the_payload_and_its_name_from_the_definition() {
    let motion: ArchiveOmfMotion = ArchiveOmfMotion::of(&definition(0, 1.0), &payload("stale", 45), &[]);

    assert_eq!(motion.name, "idle");
    assert_eq!(motion.frames, 45);
    assert_eq!(motion.duration_seconds, 1.5);
    assert!(motion.has_diverging_label);
  }

  #[test]
  fn playback_time_is_the_span_divided_by_the_speed_the_engine_reads() {
    let motion: ArchiveOmfMotion = ArchiveOmfMotion::of(&definition(0, 2.0), &payload("idle", 60), &[]);

    assert_eq!(motion.duration_seconds, 2.0);
    assert!(
      motion
        .playback_seconds
        .is_some_and(|seconds| (seconds - 1.0).abs() < 0.01)
    );
  }

  #[test]
  fn a_motion_the_engine_reads_no_speed_for_has_no_playback_time() {
    let motion: ArchiveOmfMotion = ArchiveOmfMotion::of(&definition(0, 0.0), &payload("idle", 60), &[]);

    assert_eq!(motion.playback_seconds, None);
  }

  #[test]
  fn the_flag_word_decides_whether_the_target_is_a_part_or_a_bone() {
    let mut fx: SkeletonMotionDefinition = definition(1, 1.0);

    fx.bone_or_part = 0;

    assert_eq!(
      ArchiveOmfMotion::of(&fx, &payload("idle", 1), &[]).target,
      ArchiveOmfTarget::Bone { index: 0, name: None }
    );

    let mut cycle: SkeletonMotionDefinition = definition(0, 1.0);

    cycle.bone_or_part = 0;

    assert_eq!(
      ArchiveOmfMotion::of(&cycle, &payload("idle", 1), &[]).target,
      ArchiveOmfTarget::Part { index: 0, name: None }
    );
  }

  #[test]
  fn only_the_bits_a_definition_carries_are_named() {
    let motion: ArchiveOmfMotion = ArchiveOmfMotion::of(&definition(0b1_0000_0010, 1.0), &payload("idle", 1), &[]);

    assert_eq!(motion.flags, vec![String::from("esmStopAtEnd")]);
    assert_eq!(motion.unnamed_flags, 0b1_0000_0000);
  }

  #[test]
  fn marks_are_carried_with_the_intervals_they_declare() {
    let mut definition: SkeletonMotionDefinition = definition(0, 1.0);

    definition.marks = vec![SkeletonMotionMark {
      name: String::from("Left foot"),
      terminator: String::from("\r\n"),
      intervals: vec![(0.1, 0.2)],
    }];

    let motion: ArchiveOmfMotion = ArchiveOmfMotion::of(&definition, &payload("idle", 1), &[]);

    assert_eq!(motion.marks.len(), 1);
    assert_eq!(motion.marks[0].name, "Left foot");
    assert_eq!(
      motion.marks[0].intervals,
      vec![ArchiveOmfMarkInterval { from: 0.1, to: 0.2 }]
    );
  }
}
