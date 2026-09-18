use serde::Serialize;
use xrf_db::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_ogf::{OgfBone, OgfBoneIkData};
use xrf_skeleton::{SAMPLE_FPS, SkeletonBoneMotion, SkeletonMotion, SkeletonMotionDefinition, SkeletonPart};

use crate::data::visual_description::VisualTransform;
use crate::pack::visual_transform::BindTransform;

/// What one baked motion is, beside the frames themselves.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualMotionBake {
  pub name: String,
  /// Frames the buffer holds: the longest key stream the payload carries, not the count the motion declares.
  pub frame_count: u32,
  pub bone_count: u32,
  /// Seconds playing the motion takes: its frames at the format's sample rate, over its playback speed.
  pub duration: f32,
  /// The playback speed the motion's definition declares, as stored.
  pub speed: f32,
  /// How many bones the motion actually drives, the rest holding their bind pose.
  pub animated_bone_count: u32,
  /// Floats one bone's transform occupies in the baked buffer, so a consumer indexes it without agreeing a constant.
  pub floats_per_bone: u32,
}

/// Bone transforms of one baked motion, frame major: frame 0's bones, then frame 1's.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualMotionPose {
  pub description: VisualMotionBake,
  pub transforms: Vec<f32>,
}

/// Floats one baked bone transform occupies: three basis vectors and a translation.
pub const FLOATS_PER_BONE: usize = 12;

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
  let parents: Vec<Option<usize>> = bones.iter().map(|bone| find_bone(bones, &bone.parent)).collect();
  let bind_locals: Vec<BindTransform> = binds
    .iter()
    .map(|it| BindTransform::from_bind(&it.bind_rotation, &it.bind_position))
    .collect();

  let frame_count: usize = posed_frame_count(&runs);
  let mut transforms: Vec<f32> = Vec::with_capacity(frame_count * bones.len() * FLOATS_PER_BONE);

  for frame in 0..frame_count {
    let locals: Vec<BindTransform> = (0..bones.len())
      .map(|index| match animated[index] {
        Some(run) => BindTransform::from_key(&runs[run].get_rotation(frame), runs[run].get_translation(frame)),
        None => bind_locals[index].clone(),
      })
      .collect();

    for transform in compose_chain(&locals, &parents) {
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
    description: VisualMotionBake {
      name: definition.name.clone(),
      frame_count: frame_count as u32,
      bone_count: bones.len() as u32,
      duration: frame_count as f32 / (SAMPLE_FPS * definition.get_playback_speed()),
      speed: definition.speed,
      animated_bone_count: animated.iter().filter(|it| it.is_some()).count() as u32,
      floats_per_bone: FLOATS_PER_BONE as u32,
    },
    transforms,
  })
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

      if let Some(bone) = bones.iter().position(|it| it.name == *name) {
        animated[bone] = Some(run);
      }
    }
  }

  animated
}

/// Composes local transforms into model space, root downwards.
fn compose_chain(locals: &[BindTransform], parents: &[Option<usize>]) -> Vec<Option<BindTransform>> {
  let mut model: Vec<Option<BindTransform>> = vec![None; locals.len()];

  loop {
    let mut placed: usize = 0;

    for index in 0..locals.len() {
      if model[index].is_some() {
        continue;
      }

      let resolved: Option<BindTransform> = match parents[index] {
        None => Some(locals[index].then(&BindTransform::identity())),
        Some(parent) => model[parent].as_ref().map(|it| locals[index].then(it)),
      };

      if let Some(resolved) = resolved {
        model[index] = Some(resolved);
        placed += 1;
      }
    }

    if placed == 0 {
      return model;
    }
  }
}

fn find_bone(bones: &[OgfBone], name: &str) -> Option<usize> {
  if name.is_empty() {
    return None;
  }

  bones.iter().position(|it| it.name == name)
}
