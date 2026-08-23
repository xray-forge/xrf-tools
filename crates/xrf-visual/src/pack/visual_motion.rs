use serde::Serialize;
use xrf_db::{OgfBone, OgfBoneIkData, OgfBoneMotion, OgfMotion, OgfPart, SAMPLE_FPS, Vector3d, XRayByteOrder};
use xrf_error::{XrfError, XrfResult};

use crate::pack::visual_conversion::convert_vector;
use crate::pack::visual_transform::BindTransform;

/// One motion's joint positions, every frame baked, ready to play without asking for more.
///
/// Baked rather than sampled on demand because playback runs at thirty frames a second and every frame would otherwise
/// be a round trip. A measured motion averages 78 frames, so a 47 bone skeleton bakes to about 44 kilobytes - cheaper
/// to send once than to ask for repeatedly.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VisualMotionBake {
  pub name: String,
  pub frame_count: u32,
  pub bone_count: u32,
  /// Seconds the motion runs for, from the format's fixed sample rate.
  pub duration: f32,
  /// How many bones the motion actually drives, the rest holding their bind pose.
  pub animated_bone_count: u32,
}

/// Joint positions of one baked motion, frame major: frame 0's bones, then frame 1's, three floats each.
///
/// Kept beside the description rather than inside it because it crosses to a renderer as raw bytes, the same split the
/// geometry buffer uses.
#[derive(Clone, Debug, PartialEq)]
pub struct VisualMotionPositions {
  pub description: VisualMotionBake,
  pub positions: Vec<f32>,
}

/// Bakes every frame of one motion into model-space joint positions.
///
/// Composed in engine space and converted only at the end, exactly as the bind pose is: mirroring a position is one
/// sign, while mirroring a rotation is a similarity transform that is easy to get subtly wrong. Since a skeleton
/// overlay needs positions and nothing else, the rotations never have to leave engine space at all.
///
/// A bone the motion does not drive keeps its bind transform rather than snapping to the origin, which is what the
/// engine's remap leaves untouched for a bone no partition names.
///
/// # Errors
///
/// Returns an error when the motion's key payload does not match the skeleton, which the decoder detects as bytes left
/// over rather than as misassigned keys.
pub fn bake_motion(
  bones: &[OgfBone],
  binds: &[OgfBoneIkData],
  parts: &[OgfPart],
  motion: &OgfMotion,
) -> XrfResult<VisualMotionPositions> {
  if binds.len() != bones.len() {
    return Err(XrfError::new_parsing_error(format!(
      "Motion '{}' cannot be posed: {} bones carry {} bind records",
      motion.name,
      bones.len(),
      binds.len()
    )));
  }

  let runs: Vec<OgfBoneMotion> = motion.decode_bone_motions::<XRayByteOrder>(total_part_bones(parts))?;
  // Run index to bone index, by name: the payload is ordered by the motion's own indices, and the engine resolves each
  // through `find_bone_id` (`SkeletonMotions.cpp:106`). Positional pairing would animate the wrong bones whenever a
  // partition orders them differently from the bone chunk, which is the normal case.
  let animated: Vec<Option<usize>> = resolve_animated_bones(bones, parts, runs.len());
  let parents: Vec<Option<usize>> = bones.iter().map(|bone| find_bone(bones, &bone.parent)).collect();
  let bind_locals: Vec<BindTransform> = binds
    .iter()
    .map(|it| BindTransform::from_bind(&it.bind_rotation, &it.bind_position))
    .collect();

  let frame_count: usize = motion.count.max(1) as usize;
  let mut positions: Vec<f32> = Vec::with_capacity(frame_count * bones.len() * 3);

  for frame in 0..frame_count {
    let locals: Vec<BindTransform> = (0..bones.len())
      .map(|index| match animated[index] {
        Some(run) => BindTransform::from_key(&runs[run].get_rotation(frame), runs[run].get_translation(frame)),
        None => bind_locals[index].clone(),
      })
      .collect();

    for transform in compose_chain(&locals, &parents) {
      let converted: Vector3d = match transform {
        Some(transform) => convert_vector(&transform.c),
        // A bone whose chain never reaches a root has no place to be drawn; the origin is the only honest answer, and
        // the segment it would draw is dropped by the consumer anyway.
        None => Vector3d { x: 0.0, y: 0.0, z: 0.0 },
      };

      positions.extend([converted.x, converted.y, converted.z]);
    }
  }

  Ok(VisualMotionPositions {
    description: VisualMotionBake {
      name: motion.name.clone(),
      frame_count: frame_count as u32,
      bone_count: bones.len() as u32,
      duration: frame_count as f32 / SAMPLE_FPS,
      animated_bone_count: animated.iter().filter(|it| it.is_some()).count() as u32,
    },
    positions,
  })
}

/// Bones the partitions name in total, which is how many key runs the payload holds.
pub fn total_part_bones(parts: &[OgfPart]) -> usize {
  parts.iter().map(|it| it.bones.len()).sum()
}

/// Which bone each key run drives, resolved by name.
///
/// Indexed by bone rather than by run, because that is the order a pose is composed in. A partition naming a bone the
/// skeleton does not have contributes nothing rather than failing: an omf shared between similar models is normal, and
/// the bones it does match still animate.
fn resolve_animated_bones(bones: &[OgfBone], parts: &[OgfPart], run_count: usize) -> Vec<Option<usize>> {
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
///
/// Iterative for the same reason the bind pose is: the format does not promise parents precede children, and a cycle
/// has to terminate rather than recurse forever.
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
