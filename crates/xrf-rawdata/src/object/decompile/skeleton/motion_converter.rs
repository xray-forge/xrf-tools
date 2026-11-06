use byteorder::ByteOrder;
use xrf_animation_envelope::{AnimationEnvelope, AnimationKey, AnimationShape};
use xrf_editable_object::{EditableBoneMotion, EditableBonePart, EditableMotion};
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Affine3d, Quaternion, Vector3d};
use xrf_ogf::OgfFile;
use xrf_skeleton::{SkeletonBoneMotion, SkeletonMotion, SkeletonMotionDefinition, SkeletonMotionParametersChunk};

/// Rebuilds the motions an OGF keeps itself into the object's own, which `CExportSkeleton::ExportMotions` sampled
/// them from: each bone keyed at every frame, linearly between them, and the bone parts they play on.
///
/// What the keys cannot give back is inferred: the authored curves between frames, and which bones kept their world
/// orientation, since a key holds the pose the bone took under its parent.
pub(crate) struct MotionConverter;

/// What [`MotionConverter`] rebuilds.
pub(crate) struct ConvertedMotions {
  pub(crate) motions: Vec<EditableMotion>,
  pub(crate) parts: Vec<EditableBonePart>,
}

impl MotionConverter {
  /// `SAMPLE_FPS`, the rate every OGF motion is sampled at.
  const FPS: f32 = SkeletonMotion::SAMPLE_FPS;

  /// The OGF's own motions, none where it stores none.
  ///
  /// # Errors
  ///
  /// Returns an error when the motions come without the parameters that say how they play, a part names a bone the
  /// skeleton lacks, or a motion's keys do not decode for the skeleton's bones.
  pub(crate) fn convert<T: ByteOrder>(file: &OgfFile, bone_names: &[String]) -> XrfResult<Option<ConvertedMotions>> {
    let Some(motions) = file.motions.as_ref().filter(|chunk| !chunk.motions.is_empty()) else {
      return Ok(None);
    };
    let parameters: &SkeletonMotionParametersChunk = file
      .motion_parameters
      .as_ref()
      .ok_or_else(|| XrfError::new_invalid_error("An OGF keeps motions but no parameters saying how they play"))?;
    let bone_of_run: Vec<usize> = Self::bone_of_run(parameters, bone_names)?;

    Ok(Some(ConvertedMotions {
      motions: motions
        .motions
        .iter()
        .enumerate()
        .map(|(index, motion)| {
          let definition: Option<&SkeletonMotionDefinition> = parameters
            .motions
            .iter()
            .find(|definition| usize::from(definition.motion) == index);

          Self::convert_motion::<T>(motion, definition, &bone_of_run, bone_names)
        })
        .collect::<XrfResult<_>>()?,
      parts: parameters
        .parts
        .iter()
        .map(|part| EditableBonePart {
          alias: part.name.clone(),
          bones: part.bones.iter().map(|(name, _)| name.clone()).collect(),
        })
        .collect(),
    }))
  }

  /// The bone each key run animates: `motions_value::load` hands run `i` to the bone a part stores with index `i`.
  fn bone_of_run(parameters: &SkeletonMotionParametersChunk, bone_names: &[String]) -> XrfResult<Vec<usize>> {
    let mut bone_of_run: Vec<usize> = (0..bone_names.len()).collect();

    for (name, run) in parameters.parts.iter().flat_map(|part| &part.bones) {
      let bone: usize = bone_names
        .iter()
        .position(|it| it.eq_ignore_ascii_case(name))
        .ok_or_else(|| XrfError::new_invalid_error(format!("A bone part names bone '{name}', which has no bone")))?;

      if let Some(slot) = bone_of_run.get_mut(*run as usize) {
        *slot = bone;
      }
    }

    Ok(bone_of_run)
  }

  fn convert_motion<T: ByteOrder>(
    motion: &SkeletonMotion,
    definition: Option<&SkeletonMotionDefinition>,
    bone_of_run: &[usize],
    bone_names: &[String],
  ) -> XrfResult<EditableMotion> {
    let frame_end: i32 = i32::try_from(motion.count)
      .ok()
      .and_then(|count| count.checked_sub(1))
      .filter(|end| *end >= 0)
      .ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Motion '{}' declares {} frames, where the editor keys one to {}",
          motion.label,
          motion.count,
          i32::MAX
        ))
      })?;
    let runs: Vec<SkeletonBoneMotion> = motion.decode_bone_motions::<T>(bone_names.len())?;
    let mut bones: Vec<Option<EditableBoneMotion>> = vec![None; bone_names.len()];

    for (run, keys) in runs.iter().enumerate() {
      let bone: usize = bone_of_run.get(run).copied().unwrap_or(run);

      bones[bone] = Some(Self::convert_bone(&bone_names[bone], keys, motion.count as usize));
    }

    // `CSMotion`'s own defaults, for a motion no definition describes.
    let mut converted: EditableMotion = EditableMotion {
      name: motion.label.clone(),
      frame_start: 0,
      frame_end,
      fps: Self::FPS,
      version: EditableMotion::CURRENT_VERSION,
      flags: 0,
      bone_or_part: u16::MAX,
      speed: 1.0,
      accrue: 2.0,
      falloff: 2.0,
      power: 1.0,
      bones: bones.into_iter().flatten().collect(),
      marks: Vec::new(),
    };

    if let Some(definition) = definition {
      converted.name = definition.name.clone();
      converted.flags = definition.flags as u8;
      converted.bone_or_part = definition.bone_or_part;
      converted.speed = definition.speed;
      converted.accrue = definition.accrue;
      converted.falloff = definition.falloff;
      converted.power = definition.power;
      converted.marks = definition.marks.clone();
    }

    Ok(converted)
  }

  /// A bone's channels through its keys: the rotation as the bone angles `setXYZi` takes, each angle carried on by
  /// whole turns so it never jumps between frames, and the position. A bone the motion holds still, one key of each,
  /// is read once, whatever frame count the motion declares.
  fn convert_bone(name: &str, keys: &SkeletonBoneMotion, frames: usize) -> EditableBoneMotion {
    let is_held: bool = keys.rotations.len() <= 1 && keys.translations.len() <= 1;
    let mut values: [Vec<f32>; 6] = Default::default();

    for frame in 0..if is_held { 1 } else { frames.max(1) } {
      let rotation: Quaternion = keys.get_rotation(frame).normalize_or_identity();
      let angles: Vector3d = Affine3d::from_rotation(&rotation).get_xyz_inverse();
      let position: &Vector3d = keys.get_translation(frame);
      // Heading is the angles' `y` and pitch their `x`, as the export reads them back.
      let frame_values: [f32; 6] = [position.x, position.y, position.z, angles.y, angles.x, angles.z];

      for (channel, value) in values.iter_mut().zip(frame_values) {
        channel.push(value);
      }
    }

    for channel in &mut values[EditableBoneMotion::ROTATION_HEADING..] {
      Self::unwrap(channel);
    }

    EditableBoneMotion {
      name: String::from(name),
      flags: 0,
      channels: values.map(|channel| Self::envelope(&channel)),
    }
  }

  /// Angles moved by whole turns to lie within half a turn of the one before, so the curve between frames takes the
  /// short way.
  fn unwrap(angles: &mut [f32]) {
    use std::f32::consts::{PI, TAU};

    for index in 1..angles.len() {
      let previous: f32 = angles[index - 1];

      angles[index] -= TAU * ((angles[index] - previous + PI) / TAU).floor();
    }
  }

  /// One key where every frame holds the same value, else a linear key at each frame's time, `frame / SAMPLE_FPS` as
  /// the export samples it.
  fn envelope(values: &[f32]) -> AnimationEnvelope {
    let mut envelope: AnimationEnvelope = AnimationEnvelope::default();

    if values.iter().all(|value| *value == values[0]) {
      envelope.insert_key(0.0, values[0]);

      return envelope;
    }

    envelope.keys = values
      .iter()
      .enumerate()
      .map(|(frame, value)| AnimationKey {
        value: *value,
        time: frame as f32 / Self::FPS,
        shape: AnimationShape::Linear,
        ..AnimationKey::default()
      })
      .collect();

    envelope
  }
}
