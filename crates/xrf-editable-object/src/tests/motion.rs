use xrf_animation_envelope::AnimationEnvelope;
use xrf_chunk::{ChunkReader, ChunkWriter, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_skeleton::SkeletonMotionMark;

use crate::{EditableBoneMotion, EditableMotion};

/// A channel through `values` at one key a second, as storing it leaves it: the narrow layout quantises its keys'
/// parameters.
fn channel(values: &[f32]) -> AnimationEnvelope {
  let mut channel: AnimationEnvelope = AnimationEnvelope::default();

  for (second, value) in values.iter().enumerate() {
    channel.insert_key(second as f32, *value);
  }

  let mut writer: ChunkWriter = ChunkWriter::new();

  channel.write::<XRayByteOrder>(&mut writer).expect("the channel writes");

  let bytes: Vec<u8> = writer.flush_raw_into_buffer().expect("the channel writes");

  AnimationEnvelope::read::<XRayByteOrder, _>(&mut ChunkReader::from_bytes(&bytes).expect("the channel reads"))
    .expect("the channel reads")
}

fn bone(name: &str, flags: u8) -> EditableBoneMotion {
  EditableBoneMotion {
    name: String::from(name),
    flags,
    channels: [
      channel(&[0.0, 1.0]),
      channel(&[0.5]),
      channel(&[]),
      channel(&[0.0, 0.25, 0.5]),
      channel(&[0.0]),
      channel(&[-1.0, 1.0]),
    ],
  }
}

fn motion(marks: Vec<SkeletonMotionMark>) -> EditableMotion {
  EditableMotion {
    name: String::from("open"),
    frame_start: 0,
    frame_end: 30,
    fps: 30.0,
    version: EditableMotion::CURRENT_VERSION,
    flags: EditableMotion::STOP_AT_END_FLAG,
    bone_or_part: u16::MAX,
    speed: 1.0,
    accrue: 2.0,
    falloff: 2.0,
    power: 1.0,
    bones: vec![bone("link", 0), bone("door", EditableBoneMotion::WORLD_ORIENT_FLAG)],
    marks,
  }
}

/// `motion` written, with its version field and the marks after its last bone changed as an older SDK stored them.
fn written_as(version: u16, motion: &EditableMotion) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = EditableMotion::write_list::<XRayByteOrder>(std::slice::from_ref(motion))?;
  // The count, then the name and its terminator, then the first and last frames and the rate.
  let at: usize = 4 + motion.name.len() + 1 + 12;

  bytes[at..at + 2].copy_from_slice(&version.to_le_bytes());
  bytes.truncate(bytes.len() - 4);

  Ok(bytes)
}

#[test]
fn reads_back_the_motions_it_writes() -> XrfResult {
  let motions: Vec<EditableMotion> = vec![
    motion(vec![SkeletonMotionMark {
      name: String::from("unlock"),
      terminator: String::from("\r\n"),
      intervals: vec![(0.25, 0.5)],
    }]),
    motion(Vec::new()),
  ];

  assert_eq!(
    EditableMotion::read_list::<XRayByteOrder>(&EditableMotion::write_list::<XRayByteOrder>(&motions)?)?,
    motions
  );
  assert_eq!(motions[0].get_length(), 31);
  assert!(motions[0].bones[1].is_world_oriented());

  Ok(())
}

#[test]
fn reads_a_version_6_motion_which_stores_no_marks() -> XrfResult {
  let read: Vec<EditableMotion> = EditableMotion::read_list::<XRayByteOrder>(&written_as(6, &motion(Vec::new()))?)?;

  assert_eq!(
    read,
    vec![EditableMotion {
      version: 6,
      ..motion(Vec::new())
    }]
  );

  Ok(())
}

#[test]
fn refuses_a_version_the_sdk_does_not_read() -> XrfResult {
  let error: String = EditableMotion::read_list::<XRayByteOrder>(&written_as(3, &motion(Vec::new()))?)
    .expect_err("expect version 3 to be refused")
    .to_string();

  assert!(error.contains("version 3"), "Unexpected error: {error}");

  Ok(())
}
