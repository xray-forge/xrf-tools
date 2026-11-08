use byteorder::ByteOrder;
use xrf_animation_envelope::{AnimationEnvelope, AnimationKey, AnimationShape};
use xrf_editable_object::{EditableBoneMotion, EditableBonePart, EditableMotion};
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Quaternion, Vector3d};
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
  /// Whether a motion is as long as its keys rather than as it declares (see [`MotionConverter::decode_runs`]).
  pub(crate) is_lengthened: bool,
}

impl MotionConverter {
  /// `SAMPLE_FPS`, the rate every OGF motion is sampled at.
  const FPS: f32 = SkeletonMotion::SAMPLE_FPS;

  /// The OGF's own motions, none where it stores none.
  ///
  /// # Errors
  ///
  /// Returns an error when the motions come without the parameters that say how they play, a part names a bone the
  /// skeleton lacks, or a motion's keys do not decode for the skeleton's bones at its declared length or one key more.
  pub(crate) fn convert<T: ByteOrder>(file: &OgfFile, bone_names: &[String]) -> XrfResult<Option<ConvertedMotions>> {
    let Some(motions) = file.motions.as_ref().filter(|chunk| !chunk.motions.is_empty()) else {
      return Ok(None);
    };
    let parameters: &SkeletonMotionParametersChunk = file
      .motion_parameters
      .as_ref()
      .ok_or_else(|| XrfError::new_invalid_error("An OGF keeps motions but no parameters saying how they play"))?;
    let bone_of_run: Vec<usize> = Self::bone_of_run(parameters, bone_names)?;
    let mut is_lengthened: bool = false;
    let mut converted: Vec<EditableMotion> = Vec::with_capacity(motions.motions.len());

    for (index, motion) in motions.motions.iter().enumerate() {
      let definition: Option<&SkeletonMotionDefinition> = parameters
        .motions
        .iter()
        .find(|definition| usize::from(definition.motion) == index);
      let (runs, frames): (Vec<SkeletonBoneMotion>, u32) = Self::decode_runs::<T>(motion, bone_names.len())?;

      is_lengthened |= frames != motion.count;
      converted.push(Self::convert_motion(
        motion,
        &runs,
        frames,
        definition,
        &bone_of_run,
        bone_names,
      )?);
    }

    Ok(Some(ConvertedMotions {
      motions: converted,
      parts: parameters
        .parts
        .iter()
        .map(|part| EditableBonePart {
          alias: part.name.clone(),
          bones: part.bones.iter().map(|(name, _)| name.clone()).collect(),
        })
        .collect(),
      is_lengthened,
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

  /// A motion's key runs, one per bone, and the frames they span: those it declares, or one more where every run holds
  /// one key more and only so do the runs fill the payload exactly.
  ///
  /// `blender-xray` declares `int(end - start + 1)` frames but samples `int(start)` to `int(end)`, one frame more for an
  /// action starting between frames; `motions_value::load` then reads each run short, and the rest of the payload as
  /// the next runs. The keys are whole, so the motion is read at their length, and compiled again it declares it.
  ///
  /// # Errors
  ///
  /// Returns the declared length's error when the keys decode at neither length.
  fn decode_runs<T: ByteOrder>(motion: &SkeletonMotion, bones: usize) -> XrfResult<(Vec<SkeletonBoneMotion>, u32)> {
    let error: XrfError = match motion.decode_bone_motions::<T>(bones) {
      Ok(runs) => return Ok((runs, motion.count)),
      Err(error) => error,
    };
    let Some(frames) = motion.count.checked_add(1) else {
      return Err(error);
    };
    let lengthened: SkeletonMotion = SkeletonMotion {
      count: frames,
      ..motion.clone()
    };

    lengthened
      .decode_bone_motions::<T>(bones)
      .map(|runs| (runs, frames))
      .map_err(|_| error)
  }

  fn convert_motion(
    motion: &SkeletonMotion,
    runs: &[SkeletonBoneMotion],
    frames: u32,
    definition: Option<&SkeletonMotionDefinition>,
    bone_of_run: &[usize],
    bone_names: &[String],
  ) -> XrfResult<EditableMotion> {
    let frame_end: i32 = i32::try_from(frames)
      .ok()
      .and_then(|count| count.checked_sub(1))
      .filter(|end| *end >= 0)
      .ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Motion '{}' declares {} frames, where the editor keys one to {}",
          motion.label,
          frames,
          i32::MAX
        ))
      })?;
    let mut bones: Vec<Option<EditableBoneMotion>> = vec![None; bone_names.len()];

    for (run, keys) in runs.iter().enumerate() {
      let bone: usize = bone_of_run.get(run).copied().unwrap_or(run);

      bones[bone] = Some(Self::convert_bone(&bone_names[bone], keys, frames as usize));
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
      let angles: Vector3d = Self::get_xyz_inverse(&keys.get_rotation(frame));
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

  /// The bone angles a key's rotation stands for, `Fmatrix::getXYZi` of its matrix (`Fmatrix::rotation`), worked in
  /// double precision on the normalised quaternion; the identity's for a quaternion of no length.
  ///
  /// By a straight up or down pitch, heading and bank come from matrix elements as small as 1e-5, which a 16-bit key's
  /// rounding and single precision leave mostly noise: worked in `f32`, keys within a degree of the pole came back up
  /// to 73 quantisation steps off. In double the elements are exact, and so are the angles to single precision.
  pub(crate) fn get_xyz_inverse(rotation: &Quaternion) -> Vector3d {
    let [x, y, z, w] = [rotation.x, rotation.y, rotation.z, rotation.w].map(f64::from);
    let length: f64 = (x * x + y * y + z * z + w * w).sqrt();

    if length == 0.0 {
      return Vector3d::ZERO;
    }

    let [x, y, z, w] = [x, y, z, w].map(|component| component / length);
    // `Affine3d::from_rotation`'s terms, those `getHPB` reads.
    let (x_axis_x, x_axis_y, x_axis_z) = (
      1.0 - 2.0 * (y * y + z * z),
      2.0 * (x * y - w * z),
      2.0 * (x * z + w * y),
    );
    let y_axis_y: f64 = 1.0 - 2.0 * (x * x + z * z);
    let (z_axis_x, z_axis_y, z_axis_z) = (
      2.0 * (x * z - w * y),
      2.0 * (y * z + w * x),
      1.0 - 2.0 * (x * x + y * y),
    );
    let cos_pitch: f64 = (y_axis_y * y_axis_y + x_axis_y * x_axis_y).sqrt();
    let pitch: f64 = -(-z_axis_y).atan2(cos_pitch);
    let (heading, bank): (f64, f64) = if cos_pitch > 16.0 * f64::EPSILON {
      (-z_axis_x.atan2(z_axis_z), -x_axis_y.atan2(y_axis_y))
    } else {
      (-(-x_axis_z).atan2(x_axis_x), 0.0)
    };

    // `getXYZi` negates `getHPB` and swaps heading and pitch.
    Vector3d::new(-pitch as f32, -heading as f32, -bank as f32)
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
