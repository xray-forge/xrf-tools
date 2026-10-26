use xrf_error::{XrfError, XrfResult};
use xrf_ogf::{OgfBone, OgfBoneIkData};
use xrf_skeleton::{SkeletonBoneMotion, SkeletonMotion, SkeletonMotionDefinition, SkeletonPart};
use xrf_spawn::XRayByteOrder;

use crate::data::visual::skeleton::bind_transform::BindTransform;
use crate::data::visual::skeleton::visual_transform::VisualTransform;
use crate::pack::visual::motion::visual_motion_bake::VisualMotionBake;
use crate::pack::visual::motion::visual_motion_pose::VisualMotionPose;
use crate::pack::visual::visual_skeleton::VisualSkeleton;

/// Floats one baked bone transform occupies: three basis vectors and a translation.
pub const FLOATS_PER_BONE: usize = VisualTransform::FLOATS;

/// Bakes every frame of one motion into model-space bone transforms.
///
/// # Errors
///
/// Returns an error when the motion's key payload does not match the skeleton, which the decoder detects as bytes left
/// over rather than as misassigned keys.
pub fn bake_motion(
  bones: &[OgfBone],
  binds: &[OgfBoneIkData],
  parts: &[SkeletonPart],
  definition: &SkeletonMotionDefinition,
  motion: &SkeletonMotion,
) -> XrfResult<VisualMotionPose> {
  let (runs, animated) = decode_runs(bones, binds, parts, definition, motion)?;
  let parents: Vec<Option<usize>> = VisualSkeleton::resolve_parents(bones);
  let bind_locals: Vec<BindTransform> = binds
    .iter()
    .map(|it| BindTransform::from_bind(&it.bind_rotation, &it.bind_position))
    .collect();

  let description: VisualMotionBake = describe_runs(bones.len(), definition, &runs, &animated);
  let frame_count: usize = description.frame_count as usize;
  let mut transforms: Vec<f32> = Vec::with_capacity(frame_count * bones.len() * FLOATS_PER_BONE);

  for frame in 0..frame_count {
    let locals: Vec<BindTransform> = (0..bones.len())
      .map(|index| match animated[index] {
        Some(run) => BindTransform::from_key(&runs[run].get_rotation(frame), runs[run].get_translation(frame)),
        None => bind_locals[index].clone(),
      })
      .collect();

    for transform in BindTransform::compose_chain(&locals, &parents) {
      // A bone whose chain never reaches a root has no place to be posed; the identity is the only honest answer, and
      // it leaves whatever is skinned to that bone sitting in model space rather than collapsed to a point.
      let posed: VisualTransform = transform
        .as_ref()
        .unwrap_or(&BindTransform::identity())
        .to_renderer_space();

      transforms.extend([
        posed.i.x, posed.i.y, posed.i.z, posed.j.x, posed.j.y, posed.j.z, posed.k.x, posed.k.y, posed.k.z, posed.c.x,
        posed.c.y, posed.c.z,
      ]);
    }
  }

  Ok(VisualMotionPose {
    description,
    transforms: transforms.into(),
  })
}

/// Describes one motion as `bake_motion` bakes it, its frames, bones and length, decoding its keys but posing no frame.
///
/// # Errors
///
/// Returns an error where `bake_motion` would: the motion's key payload does not match the skeleton.
pub fn describe_motion(
  bones: &[OgfBone],
  binds: &[OgfBoneIkData],
  parts: &[SkeletonPart],
  definition: &SkeletonMotionDefinition,
  motion: &SkeletonMotion,
) -> XrfResult<VisualMotionBake> {
  let (runs, animated) = decode_runs(bones, binds, parts, definition, motion)?;

  Ok(describe_runs(bones.len(), definition, &runs, &animated))
}

/// A motion's key runs, and the run each bone is driven by.
fn decode_runs(
  bones: &[OgfBone],
  binds: &[OgfBoneIkData],
  parts: &[SkeletonPart],
  definition: &SkeletonMotionDefinition,
  motion: &SkeletonMotion,
) -> XrfResult<(Vec<SkeletonBoneMotion>, Vec<Option<usize>>)> {
  if binds.len() != bones.len() {
    return Err(XrfError::new_parsing_error(format!(
      "Motion '{}' cannot be posed: {} bones carry {} bind records",
      definition.name,
      bones.len(),
      binds.len()
    )));
  }

  let runs: Vec<SkeletonBoneMotion> = motion
    .decode_bone_motions::<XRayByteOrder>(total_part_bones(parts))
    .map_err(|error| XrfError::new_parsing_error(format!("Motion '{}' cannot be posed: {error}", definition.name)))?;

  // Run index to bone index, by name: the payload is ordered by the motion's own indices, and the engine resolves each
  // through `find_bone_id` (`SkeletonMotions.cpp:106`). Positional pairing would animate the wrong bones whenever a
  // partition orders them differently from the bone chunk, which is the normal case.
  let animated: Vec<Option<usize>> = resolve_animated_bones(bones, parts, runs.len());

  Ok((runs, animated))
}

/// What a bake of decoded runs holds, for a skeleton of `bones` bones.
fn describe_runs(
  bones: usize,
  definition: &SkeletonMotionDefinition,
  runs: &[SkeletonBoneMotion],
  animated: &[Option<usize>],
) -> VisualMotionBake {
  let frame_count: usize = posed_frame_count(runs);

  VisualMotionBake {
    name: definition.name.clone(),
    frame_count: frame_count as u32,
    bone_count: bones as u32,
    duration: frame_count as f32 / (SkeletonMotion::SAMPLE_FPS * definition.get_playback_speed()),
    speed: definition.speed,
    animated_bone_count: animated.iter().filter(|it| it.is_some()).count() as u32,
    floats_per_bone: FLOATS_PER_BONE as u32,
  }
}

/// Frames the decoded runs can tell apart, which is how many distinct poses the payload paid for.
fn posed_frame_count(runs: &[SkeletonBoneMotion]) -> usize {
  runs
    .iter()
    .map(|it| it.rotations.len().max(it.translations.len()))
    .max()
    .unwrap_or(1)
}

/// Bones the partitions name in total, which is how many key runs the payload holds.
pub fn total_part_bones(parts: &[SkeletonPart]) -> usize {
  parts.iter().map(|it| it.bones.len()).sum()
}

/// Which bone each key run drives, resolved by name.
fn resolve_animated_bones(bones: &[OgfBone], parts: &[SkeletonPart], run_count: usize) -> Vec<Option<usize>> {
  let mut animated: Vec<Option<usize>> = vec![None; bones.len()];

  for part in parts {
    for (name, run) in &part.bones {
      let run: usize = *run as usize;

      if run >= run_count {
        continue;
      }

      // Case-blind as the engine is, which lower-cases both names before `find_bone_id` compares them
      // (`SkeletonMotions.cpp`, `SkeletonCustom.cpp`).
      if let Some(bone) = bones.iter().position(|it| it.name.eq_ignore_ascii_case(name)) {
        animated[bone] = Some(run);
      }
    }
  }

  animated
}
