use xrf_animation_envelope::AnimationEnvelope;
use xrf_chunk::XRayByteOrder;
use xrf_editable_object::{EditableBoneMotion, EditableMotion, EditableObjectFile};
use xrf_error::XrfResult;
use xrf_math::{Quaternion, Vector3d};
use xrf_ogf::{OgfFile, OgfModelType};
use xrf_skeleton::{SkeletonBoneMotion, SkeletonMotion, SkeletonMotionParametersChunk};

use crate::object::compile::motion::motion_keys::MotionKeys;
use crate::object::compile::tests::ogf::{authored, compile};
use crate::object::decompile::object_decompile::ObjectDecompile;
use crate::object::decompile::object_decompiler::ObjectDecompiler;
use crate::object::decompile::object_inference::ObjectInference;

/// Decodes the one run `rotations` and `translations` write.
fn run(rotations: &[[i16; 4]], translations: &[Vector3d]) -> XrfResult<(u8, SkeletonBoneMotion)> {
  let mut keys: Vec<u8> = Vec::new();

  MotionKeys::write::<XRayByteOrder>(&mut keys, rotations, translations)?;

  let flags: u8 = keys[0];
  let motion: SkeletonMotion = SkeletonMotion {
    label: String::from("test"),
    count: translations.len() as u32,
    keys,
  };

  Ok((flags, motion.decode_bone_motions::<XRayByteOrder>(1)?.remove(0)))
}

fn along_x(values: &[f32]) -> Vec<Vector3d> {
  values.iter().map(|x| Vector3d::new(*x, 2.0, -1.0)).collect()
}

#[test]
fn holds_a_still_bone_in_one_rotation_and_its_mean_position() -> XrfResult {
  let rotation: [i16; 4] = MotionKeys::quantize_rotation(&Quaternion::IDENTITY);
  // Within a millimetre of the mean on every axis, which the export takes for not moving.
  let (flags, bone): (u8, SkeletonBoneMotion) = run(&[rotation; 3], &along_x(&[0.0, 0.0004, 0.0008]))?;

  assert_eq!(flags, 1 << 1, "expect the rotation held and no translation stream");
  assert_eq!(bone.rotations, vec![Quaternion::IDENTITY]);
  assert_eq!(bone.translations.len(), 1);
  assert!((bone.translations[0].x - 0.0004).abs() < 1e-6);

  Ok(())
}

#[test]
fn quantises_a_short_move_to_8_bits_and_a_long_one_to_16() -> XrfResult {
  let rotations: [[i16; 4]; 3] = [MotionKeys::quantize_rotation(&Quaternion::IDENTITY); 3];

  for (values, flags) in [([0.0, 0.5, 1.0], 1), ([0.0, 2.0, 4.0], 1 | 1 << 2)] {
    let (written, bone): (u8, SkeletonBoneMotion) = run(&rotations, &along_x(&values))?;
    let step: f32 = values[2] / 2.0 / if flags & 1 << 2 != 0 { 32767.0 } else { 127.0 };

    assert_eq!(written, flags | 1 << 1);

    for (decoded, value) in bone.translations.iter().zip(values) {
      assert!((decoded.x - value).abs() <= step, "{} against {value}", decoded.x);
      // An axis that never moves comes back exactly, whatever its keys hold.
      assert_eq!((decoded.y, decoded.z), (2.0, -1.0));
    }
  }

  Ok(())
}

/// [`authored`] playing a motion of its own instead of referencing motion files: `lid` rising a metre over a second,
/// `link` and `hinge` left to their bind poses.
fn animated(flags: u8) -> XrfResult<EditableObjectFile> {
  let mut object: EditableObjectFile = authored()?;
  let mut rise: AnimationEnvelope = AnimationEnvelope::default();

  rise.insert_key(0.0, 0.0);
  rise.insert_key(1.0, 1.0);

  let motion: EditableMotion = EditableMotion {
    name: String::from("open"),
    frame_start: 0,
    frame_end: 30,
    fps: 30.0,
    version: EditableMotion::CURRENT_VERSION,
    flags,
    bone_or_part: u16::MAX,
    speed: 1.0,
    accrue: 2.0,
    falloff: 2.0,
    power: 1.0,
    bones: vec![EditableBoneMotion {
      name: String::from("LID"),
      flags: 0,
      channels: [
        AnimationEnvelope::default(),
        rise,
        AnimationEnvelope::default(),
        AnimationEnvelope::default(),
        AnimationEnvelope::default(),
        AnimationEnvelope::default(),
      ],
    }],
    marks: Vec::new(),
  };

  object.motion_references = None;
  object.embedded_motions = Some(EditableMotion::write_list::<XRayByteOrder>(&[motion])?);

  Ok(object)
}

#[test]
fn samples_each_frame_of_the_objects_own_motion() -> XrfResult {
  let file: OgfFile = compile(&animated(EditableMotion::STOP_AT_END_FLAG)?)?;
  let motions: &[SkeletonMotion] = &file.motions.as_ref().expect("Expect the object's motions").motions;
  let parameters: &SkeletonMotionParametersChunk = file.motion_parameters.as_ref().expect("Expect their parameters");
  let bones: Vec<SkeletonBoneMotion> = motions[0].decode_bone_motions::<XRayByteOrder>(3)?;

  assert_eq!(file.header.model_type, OgfModelType::SkeletonAnim.to_raw());
  assert!(
    file.kinematics.is_none(),
    "expect no motion references beside the motions"
  );
  assert_eq!(
    (motions.len(), motions[0].label.as_str(), motions[0].count),
    (1, "open", 31)
  );
  // `lid` rises by its name in any case; the bones the motion leaves out hold still.
  assert!((bones[1].get_translation(15).y - 0.5).abs() < 1e-3);
  assert!((bones[1].get_translation(30).y - 1.0).abs() < 1e-3);
  assert_eq!((bones[0].translations.len(), bones[2].translations.len()), (1, 1));
  assert_eq!(parameters.parts.len(), 1);
  assert_eq!(parameters.parts[0].name, "default");
  assert_eq!(parameters.get_bones_count(), 3);
  assert_eq!(
    (parameters.motions[0].name.as_str(), parameters.motions[0].flags),
    ("open", u32::from(EditableMotion::STOP_AT_END_FLAG))
  );

  Ok(())
}

#[test]
fn decompiles_its_motions_into_keys_that_compile_alike() -> XrfResult {
  let first: OgfFile = compile(&animated(EditableMotion::STOP_AT_END_FLAG)?)?;
  let decompile: ObjectDecompile = ObjectDecompiler::decompile(&first)?;
  let second: OgfFile = compile(&decompile.object)?;
  let motions = |file: &OgfFile| file.motions.as_ref().expect("Expect motions").motions.clone();
  let parameters = |file: &OgfFile| {
    let chunk: &SkeletonMotionParametersChunk = file.motion_parameters.as_ref().expect("Expect parameters");

    (chunk.parts.clone(), chunk.motions.clone())
  };

  assert!(decompile.inferred.contains(&ObjectInference::MotionCurves));
  assert_eq!(parameters(&second), parameters(&first));

  for (motion, other) in motions(&second).iter().zip(&motions(&first)) {
    assert_eq!((&motion.label, motion.count), (&other.label, other.count));

    let runs: Vec<SkeletonBoneMotion> = motion.decode_bone_motions::<XRayByteOrder>(3)?;
    let others: Vec<SkeletonBoneMotion> = other.decode_bone_motions::<XRayByteOrder>(3)?;

    for (run, other) in runs.iter().zip(&others) {
      for frame in 0..motion.count as usize {
        let (rotation, expected) = (run.get_rotation(frame), other.get_rotation(frame));

        // A quantised key turned to angles and back lands within a step of where it was.
        for (component, expected) in [rotation.x, rotation.y, rotation.z, rotation.w]
          .into_iter()
          .zip([expected.x, expected.y, expected.z, expected.w])
        {
          assert!(
            (component - expected).abs() * 32767.0 <= 1.0,
            "{component} against {expected}"
          );
        }

        // Quantised again, a position truncates towards zero and may slip a step: half the metre `lid` rises over
        // 127 steps.
        let gap: f32 = run.get_translation(frame).distance(other.get_translation(frame));

        assert!(gap <= 0.5 / 127.0 + 1e-6, "frame {frame} moved {gap}");
      }
    }
  }

  Ok(())
}

#[test]
fn reads_a_motion_declaring_one_frame_fewer_than_its_keys_at_their_length() -> XrfResult {
  // As `blender-xray` writes an action starting between frames: 31 keys a run under a header of 30.
  let sampled: OgfFile = compile(&animated(EditableMotion::STOP_AT_END_FLAG)?)?;
  let declaring = |count: u32| -> XrfResult<OgfFile> {
    let mut file: OgfFile = compile(&animated(EditableMotion::STOP_AT_END_FLAG)?)?;

    file.motions.as_mut().expect("Expect motions").motions[0].count = count;

    Ok(file)
  };
  let decompile: ObjectDecompile = ObjectDecompiler::decompile(&declaring(30)?)?;
  let compiled: OgfFile = compile(&decompile.object)?;

  assert!(decompile.inferred.contains(&ObjectInference::MotionLengths));
  assert_eq!(compiled.motions.expect("Expect motions").motions[0].count, 31);
  assert!(
    ObjectDecompiler::decompile(&declaring(29)?).is_err(),
    "expect keys two frames longer than declared refused"
  );
  assert!(
    !ObjectDecompiler::decompile(&sampled)?
      .inferred
      .contains(&ObjectInference::MotionLengths)
  );

  Ok(())
}

#[test]
fn refuses_a_motion_moving_the_object_through_its_root() -> XrfResult {
  let error: String = compile(&animated(EditableMotion::ROOT_MOVER_FLAG)?)
    .expect_err("expect a root mover to be refused")
    .to_string();

  assert!(
    error.contains("moves the object through its root"),
    "Unexpected error: {error}"
  );

  Ok(())
}
