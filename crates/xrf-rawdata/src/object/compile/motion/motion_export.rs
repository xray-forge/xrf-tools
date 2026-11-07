use byteorder::ByteOrder;
use xrf_animation_envelope::AnimationEnvelope;
use xrf_editable_object::{EditableBone, EditableBoneMotion, EditableMotion, EditableObjectFile};
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Affine3d, Vector3d};
use xrf_skeleton::{
  SkeletonMotion, SkeletonMotionDefinition, SkeletonMotionParametersChunk, SkeletonMotionsChunk, SkeletonPart,
};

use crate::object::compile::motion::motion_keys::MotionKeys;
use crate::object::compile::skeleton::skeleton_hierarchy::SkeletonHierarchy;

/// The motions an object keeps itself, as `CExportSkeleton::ExportMotions` writes them into its OGF: every bone's
/// pose sampled at each frame and quantised (`ExportMotionKeys`), then how each motion plays and the bone parts it
/// plays on (`ExportMotionDefs`).
pub(crate) struct MotionExport {
  pub(crate) motions: SkeletonMotionsChunk,
  pub(crate) parameters: SkeletonMotionParametersChunk,
}

impl MotionExport {
  /// `xrOGF_SMParamsVersion`, the version the export writes: with marks.
  const PARAMETERS_VERSION: u16 = 4;

  /// `BI_NONE`: a motion playing on every bone rather than one part.
  const ALL_PARTS: u16 = u16::MAX;

  /// The object's own motions, none for an object that keeps none or references motion files instead, which the
  /// export then leaves to those files.
  ///
  /// # Errors
  ///
  /// Returns an error where the export refuses: a motion moving the object through its root, which the export only
  /// takes from a dedicated root bone and is not compiled yet, bone parts that do not hold every bone once, or a motion
  /// on a part past them; and where the stored motions do not read.
  pub(crate) fn of<T: ByteOrder>(object: &EditableObjectFile) -> XrfResult<Option<Self>> {
    if object.motion_references.as_ref().is_some_and(|it| !it.is_empty()) {
      return Ok(None);
    }

    let motions: Vec<EditableMotion> = object.get_embedded_motions::<T>()?;

    if motions.is_empty() {
      return Ok(None);
    }

    let bones: &[EditableBone] = object.bones.as_deref().unwrap_or_default();
    let hierarchy: SkeletonHierarchy = SkeletonHierarchy::of(bones);
    // Where the editor holds the object, which the roots' keys carry: `mGT`, rotated by heading `y`, pitch `x`.
    let placement: Affine3d = object.actor_transform.as_ref().map_or(Affine3d::IDENTITY, |transform| {
      Affine3d::from_heading_pitch_bank(transform.rotation.y, transform.rotation.x, transform.rotation.z)
        .with_translation(transform.position)
    });
    let parts: Vec<SkeletonPart> = Self::parts(object, bones)?;

    Ok(Some(Self {
      motions: SkeletonMotionsChunk {
        motions: motions
          .iter()
          .map(|motion| Self::sample::<T>(motion, bones, &hierarchy, &placement))
          .collect::<XrfResult<_>>()?,
      },
      parameters: SkeletonMotionParametersChunk {
        version: Self::PARAMETERS_VERSION,
        motions: motions
          .iter()
          .enumerate()
          .map(|(index, motion)| Self::definition(motion, index, parts.len()))
          .collect::<XrfResult<_>>()?,
        parts,
      },
    }))
  }

  /// One motion's keys: every bone posed at each frame from its first to its last, `t = frame / fps`.
  fn sample<T: ByteOrder>(
    motion: &EditableMotion,
    bones: &[EditableBone],
    hierarchy: &SkeletonHierarchy,
    placement: &Affine3d,
  ) -> XrfResult<SkeletonMotion> {
    if motion.flags & EditableMotion::ROOT_MOVER_FLAG != 0 {
      return Err(XrfError::new_not_implemented_error(format!(
        "Motion '{}' moves the object through its root, which is not compiled yet",
        motion.name
      )));
    }

    let count: u32 = u32::try_from(motion.get_length())
      .ok()
      .filter(|count| *count > 0)
      .ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Motion '{}' spans frames {} to {}, which no motion's frame count holds",
          motion.name, motion.frame_start, motion.frame_end
        ))
      })?;
    let channels: Vec<EditableBoneMotion> = Self::by_skeleton(motion, bones);
    let frames: usize = count as usize;
    let mut rotations: Vec<Vec<[i16; 4]>> = vec![Vec::with_capacity(frames); bones.len()];
    let mut translations: Vec<Vec<Vector3d>> = vec![Vec::with_capacity(frames); bones.len()];

    for frame in motion.frame_start..=motion.frame_end {
      let time: f32 = frame as f32 / motion.fps;
      let local: Vec<Affine3d> = Self::pose(&channels, hierarchy, time);

      for (bone, transform) in local.iter().enumerate() {
        let transform: Affine3d = match hierarchy.parents[bone] {
          None => placement.compose(transform),
          Some(_) => *transform,
        };

        rotations[bone].push(MotionKeys::quantize_rotation(&transform.get_rotation()));
        translations[bone].push(transform.translation);
      }
    }

    let mut keys: Vec<u8> = Vec::new();

    for (rotations, translations) in rotations.iter().zip(&translations) {
      MotionKeys::write::<T>(&mut keys, rotations, translations)?;
    }

    Ok(SkeletonMotion {
      label: motion.name.clone(),
      count,
      keys,
    })
  }

  /// Each bone's transform under its parent at a time, `CalculateAnimBone`. A bone keeping its world orientation
  /// takes its rotation in the object, and its local transform is that undone by its parent's.
  fn pose(channels: &[EditableBoneMotion], hierarchy: &SkeletonHierarchy, time: f32) -> Vec<Affine3d> {
    let local: Vec<Affine3d> = channels
      .iter()
      .map(|bone| {
        let value = |channel: usize| bone.channels[channel].evaluate(time);
        let rotation: Vector3d = Vector3d::new(
          value(EditableBoneMotion::ROTATION_PITCH),
          value(EditableBoneMotion::ROTATION_HEADING),
          value(EditableBoneMotion::ROTATION_BANK),
        );

        Affine3d::from_xyz_inverse(&rotation).with_translation(Vector3d::new(
          value(EditableBoneMotion::POSITION_X),
          value(EditableBoneMotion::POSITION_Y),
          value(EditableBoneMotion::POSITION_Z),
        ))
      })
      .collect();

    if !channels.iter().any(EditableBoneMotion::is_world_oriented) {
      return local;
    }

    let mut placed: Vec<Affine3d> = local.clone();
    let mut posed: Vec<Affine3d> = local;

    for bone in hierarchy.order() {
      let parent: Affine3d = hierarchy.parents[bone].map_or(Affine3d::IDENTITY, |parent| placed[parent]);
      let own: Affine3d = posed[bone];

      placed[bone] = parent.compose(&own);

      if channels[bone].is_world_oriented() {
        placed[bone] = Affine3d {
          translation: placed[bone].translation,
          ..own
        };
        posed[bone] = parent.inverse_rigid().compose(&own);
      }
    }

    posed
  }

  /// The motion's bones in skeleton order, `CSMotion::SortBonesBySkeleton`: each found by name ignoring case, and a
  /// bone the motion does not move held at its bind pose with the first stored bone's flags. That hold puts the bind
  /// rotation's `x` in the heading channel and `y` in pitch, so the pose it gives has those two swapped, as the SDK's
  /// does.
  fn by_skeleton(motion: &EditableMotion, bones: &[EditableBone]) -> Vec<EditableBoneMotion> {
    let held_flags: u8 = motion.bones.first().map_or(0, |bone| bone.flags);

    bones
      .iter()
      .map(|bone| {
        motion
          .bones
          .iter()
          .find(|channels| channels.name.eq_ignore_ascii_case(&bone.definition.name))
          .cloned()
          .unwrap_or_else(|| {
            let (offset, rotation): (&Vector3d, &Vector3d) = (&bone.bind_pose.offset, &bone.bind_pose.rotation);

            EditableBoneMotion {
              name: bone.definition.name.clone(),
              flags: held_flags,
              channels: [offset.x, offset.y, offset.z, rotation.x, rotation.y, rotation.z].map(Self::held),
            }
          })
      })
      .collect()
  }

  /// A channel holding one value, `CEnvelope::InsertKey` at time zero.
  fn held(value: f32) -> AnimationEnvelope {
    let mut channel: AnimationEnvelope = AnimationEnvelope::default();

    channel.insert_key(0.0, value);
    channel
  }

  /// The parts motions play on: the object's own, every bone in exactly one, or one `default` part of every bone.
  fn parts(object: &EditableObjectFile, bones: &[EditableBone]) -> XrfResult<Vec<SkeletonPart>> {
    let index_of = |name: &str| {
      bones
        .iter()
        .position(|bone| bone.definition.name.eq_ignore_ascii_case(name))
    };

    match object.bone_parts.as_deref().filter(|parts| !parts.is_empty()) {
      None => Ok(vec![SkeletonPart {
        name: String::from("default"),
        bones: bones
          .iter()
          .enumerate()
          .map(|(index, bone)| (bone.definition.name.clone(), index as u32))
          .collect(),
      }]),
      Some(parts) => {
        let mut is_placed: Vec<bool> = vec![false; bones.len()];
        let parts: Vec<SkeletonPart> = parts
          .iter()
          .map(|part| {
            Ok(SkeletonPart {
              name: part.alias.to_ascii_lowercase(),
              bones: part
                .bones
                .iter()
                .map(|name| match index_of(name) {
                  Some(index) if !std::mem::replace(&mut is_placed[index], true) => Ok((name.clone(), index as u32)),
                  _ => Err(XrfError::new_invalid_error(format!(
                    "Bone part '{}' names bone '{name}', which the skeleton lacks or another part holds",
                    part.alias
                  ))),
                })
                .collect::<XrfResult<_>>()?,
            })
          })
          .collect::<XrfResult<_>>()?;

        if is_placed.contains(&false) {
          return Err(XrfError::new_invalid_error(
            "The bone parts leave a bone in none of them, which the export refuses",
          ));
        }

        Ok(parts)
      }
    }
  }

  /// How a motion plays, `ExportMotionDefs`: its flags, part, index and blending, and its marks.
  fn definition(motion: &EditableMotion, index: usize, parts: usize) -> XrfResult<SkeletonMotionDefinition> {
    let is_fx: bool = motion.flags & EditableMotion::FX_FLAG != 0;

    if !is_fx && motion.bone_or_part != Self::ALL_PARTS && usize::from(motion.bone_or_part) >= parts {
      return Err(XrfError::new_invalid_error(format!(
        "Motion '{}' plays on part {}, past the {parts} the object holds",
        motion.name, motion.bone_or_part
      )));
    }

    Ok(SkeletonMotionDefinition {
      name: motion.name.clone(),
      flags: u32::from(motion.flags),
      bone_or_part: motion.bone_or_part,
      motion: u16::try_from(index)
        .map_err(|_| XrfError::new_invalid_error("An object holds more motions than a u16 numbers"))?,
      speed: motion.speed,
      power: motion.power,
      accrue: motion.accrue,
      falloff: motion.falloff,
      marks: motion.marks.clone(),
    })
  }
}
