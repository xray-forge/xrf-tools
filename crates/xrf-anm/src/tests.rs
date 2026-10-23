use std::io::Write;
use std::time::Duration;

use byteorder::WriteBytesExt;
use xrf_animation_envelope::{
  AnimationBehavior, AnimationEnvelope, AnimationInterpolation, AnimationKey, AnimationPlayback, AnimationShape,
};
use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::{AnmChannel, AnmFile};

/// One interpolating key, as version 5 lays it out: value, time, shape, then seven quantised parameters.
fn new_curved_key_bytes(value: f32, time: f32, quantised: u16) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&value.to_le_bytes());
  bytes.extend_from_slice(&time.to_le_bytes());
  bytes.push(0);

  for _ in 0..7 {
    bytes.extend_from_slice(&quantised.to_le_bytes());
  }

  bytes
}

/// One stepped key, which stores the shape and stops.
fn new_stepped_key_bytes(value: f32, time: f32) -> Vec<u8> {
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(&value.to_le_bytes());
  bytes.extend_from_slice(&time.to_le_bytes());
  bytes.push(4);

  bytes
}

/// An envelope: the two behaviours, the key count, then the keys.
fn new_envelope_bytes(before: u8, after: u8, keys: &[Vec<u8>]) -> Vec<u8> {
  let mut bytes: Vec<u8> = vec![before, after];

  bytes.extend_from_slice(&(keys.len() as u16).to_le_bytes());

  for key in keys {
    bytes.extend_from_slice(key);
  }

  bytes
}

/// A whole animation, framed into the one chunk the format carries. Its fourth stored envelope, keyed once at -0.25,
/// is the heading in version 5 and the pitch in version 4.
fn new_animation_bytes(version: u16) -> XrfResult<Vec<u8>> {
  let mut payload: Vec<u8> = Vec::new();

  payload.extend_from_slice(b"camera_shake\0");
  payload.extend_from_slice(&0i32.to_le_bytes());
  payload.extend_from_slice(&479i32.to_le_bytes());
  payload.extend_from_slice(&30.0f32.to_le_bytes());
  payload.extend_from_slice(&version.to_le_bytes());

  payload.extend(new_envelope_bytes(
    1,
    1,
    &[
      new_curved_key_bytes(0.0, 0.0, 32768),
      new_curved_key_bytes(1.5, 8.0, 40000),
    ],
  ));
  payload.extend(new_envelope_bytes(1, 1, &[new_stepped_key_bytes(2.0, 0.0)]));
  payload.extend(new_envelope_bytes(0, 0, &[]));
  payload.extend(new_envelope_bytes(1, 2, &[new_curved_key_bytes(-0.25, 1.0, 32768)]));
  payload.extend(new_envelope_bytes(1, 1, &[]));
  payload.extend(new_envelope_bytes(1, 1, &[]));

  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.write_all(&payload)?;
  writer.flush_chunk_into_buffer::<XRayByteOrder>(AnmFile::CHUNK_ID)
}

#[test]
fn an_animation_reads_as_the_header_and_six_channels_it_is() -> XrfResult {
  let animation: AnmFile = AnmFile::read_from_bytes::<XRayByteOrder>(new_animation_bytes(AnmFile::CURRENT_VERSION)?)?;

  assert_eq!(animation.name, "camera_shake");
  assert_eq!((animation.frame_start, animation.frame_end), (0, 479));
  assert_eq!(animation.fps, 30.0);
  assert_eq!(animation.version, 5);
  assert_eq!(animation.get_channel(AnmChannel::PositionX).keys.len(), 2);
  assert_eq!(
    animation.get_channel(AnmChannel::PositionX).behavior_before,
    AnimationBehavior::Constant
  );
  assert_eq!(
    animation.get_channel(AnmChannel::RotationHeading).behavior_after,
    AnimationBehavior::Repeat
  );
  assert!(animation.get_channel(AnmChannel::PositionZ).keys.is_empty());

  Ok(())
}

#[test]
fn an_animation_is_written_back_byte_for_byte() -> XrfResult {
  // The values do not survive a round trip and are not meant to: the quantiser's grid has 65535 steps across the
  // range, so it does not contain zero and nothing lands exactly where it started. The bytes do, which is the
  // property a writer has to hold.
  for version in AnmFile::WRITABLE_VERSIONS {
    let original: Vec<u8> = new_animation_bytes(version)?;
    let animation: AnmFile = AnmFile::read_from_bytes::<XRayByteOrder>(original.clone())?;

    assert_eq!(
      animation.write_to_bytes::<XRayByteOrder>()?,
      original,
      "version {version}"
    );
  }

  Ok(())
}

#[test]
fn a_version_four_motion_stores_its_pitch_before_its_heading() -> XrfResult {
  // `COMotion::Load` reads version 4's fourth envelope into the pitch, where version 5's is the heading; the
  // channels hold the engine's order either way, so the rotation a motion plays does not depend on its version.
  let pitch_first: AnmFile = AnmFile::read_from_bytes::<XRayByteOrder>(new_animation_bytes(4)?)?;
  let current: AnmFile = AnmFile::read_from_bytes::<XRayByteOrder>(new_animation_bytes(5)?)?;

  assert_eq!(pitch_first.get_channel(AnmChannel::RotationPitch).keys[0].value, -0.25);
  assert!(pitch_first.get_channel(AnmChannel::RotationHeading).keys.is_empty());
  assert_eq!(current.get_channel(AnmChannel::RotationHeading).keys[0].value, -0.25);
  assert!(current.get_channel(AnmChannel::RotationPitch).keys.is_empty());

  // `_Evaluate` puts the pitch in `x` and the heading in `y`.
  assert_eq!(pitch_first.evaluate(1.0).1, Vector3d::new(-0.25, 0.0, 0.0));
  assert_eq!(current.evaluate(1.0).1, Vector3d::new(0.0, -0.25, 0.0));

  Ok(())
}

#[test]
fn a_stepped_key_carries_no_curve_at_all() -> XrfResult {
  let animation: AnmFile = AnmFile::read_from_bytes::<XRayByteOrder>(new_animation_bytes(AnmFile::CURRENT_VERSION)?)?;
  let key: &AnimationKey = &animation.channels[1].keys[0];

  assert_eq!(key.shape, AnimationShape::Stepped);
  assert_eq!(
    key.interpolation,
    AnimationInterpolation::default(),
    "a stepped key stores no interpolation"
  );

  // Against the interpolating key beside it, which does.
  assert_ne!(
    animation.get_channel(AnmChannel::PositionX).keys[0].interpolation,
    AnimationInterpolation::default()
  );

  Ok(())
}

#[test]
fn a_quantised_parameter_reads_as_the_step_it_sits_on() -> XrfResult {
  let animation: AnmFile = AnmFile::read_from_bytes::<XRayByteOrder>(new_animation_bytes(AnmFile::CURRENT_VERSION)?)?;
  let interpolation: AnimationInterpolation = animation.get_channel(AnmChannel::PositionX).keys[0].interpolation;

  // 32768 of 65535 across -32..32 is just past the middle, which is the closest the grid comes to zero.
  assert!(
    (interpolation.tension - 0.000_488).abs() < 0.000_01,
    "Unexpected tension: {}",
    interpolation.tension
  );

  Ok(())
}

#[test]
fn a_version_nothing_here_reads_is_refused_by_name() -> XrfResult {
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.write_w1251_string("")?;
  writer.write_u32::<XRayByteOrder>(0)?;
  writer.write_u32::<XRayByteOrder>(1)?;
  writer.write_f32::<XRayByteOrder>(30.0)?;
  writer.write_u16::<XRayByteOrder>(2)?;

  let bytes: Vec<u8> = writer.flush_chunk_into_buffer::<XRayByteOrder>(AnmFile::CHUNK_ID)?;
  let error: String = AnmFile::read_from_bytes::<XRayByteOrder>(bytes)
    .expect_err("expect an unimplemented version to be refused")
    .to_string();

  assert!(error.contains("animation version 2"), "Unexpected error: {error}");

  Ok(())
}

#[test]
fn a_payload_holding_more_than_six_channels_is_refused() -> XrfResult {
  let mut bytes: Vec<u8> = new_animation_bytes(AnmFile::CURRENT_VERSION)?;

  // A seventh envelope is bytes the chunk carries and six reads do not consume.
  bytes.extend(new_envelope_bytes(1, 1, &[]));

  let size: u32 = (bytes.len() - 8) as u32;

  bytes[4..8].copy_from_slice(&size.to_le_bytes());

  assert!(
    AnmFile::read_from_bytes::<XRayByteOrder>(bytes).is_err(),
    "a chunk holding more than six envelopes is not an animation this reads"
  );

  Ok(())
}

#[test]
fn a_declared_key_count_past_the_payload_is_refused_before_it_is_reserved() -> XrfResult {
  let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(&[1, 1, 0xff, 0xff])?;

  let error: String = AnimationEnvelope::read::<XRayByteOrder, _>(&mut reader)
    .expect_err("expect the declared key count to exceed the payload")
    .to_string();

  assert!(error.contains("animation keys"), "Unexpected error: {error}");

  Ok(())
}

#[test]
fn a_version_this_writer_does_not_emit_is_refused_rather_than_written_as_another() -> XrfResult {
  let mut animation: AnmFile =
    AnmFile::read_from_bytes::<XRayByteOrder>(new_animation_bytes(AnmFile::CURRENT_VERSION)?)?;

  animation.version = 3;

  let mut writer: ChunkWriter = ChunkWriter::new();

  assert!(
    animation.write::<XRayByteOrder>(&mut writer).is_err(),
    "version 3 stores wider keys and is read only"
  );

  // 4 is written, its envelopes laid out as 5's and its channels in its own order.
  animation.version = 4;

  assert!(animation.write::<XRayByteOrder>(&mut ChunkWriter::new()).is_ok());

  Ok(())
}

#[test]
fn what_an_animation_is_worth_comes_from_its_frames_and_its_rate() -> XrfResult {
  let animation: AnmFile = AnmFile::read_from_bytes::<XRayByteOrder>(new_animation_bytes(AnmFile::CURRENT_VERSION)?)?;

  // 0 to 479 inclusive, which is 480 frames and not 479.
  assert_eq!(animation.get_frame_count(), 480);
  assert_eq!(animation.get_duration_seconds(), 16.0);
  assert_eq!(animation.get_keys_count(), 4);
  assert!(animation.is_keyed());

  Ok(())
}

#[test]
fn a_range_starting_before_zero_spans_the_frames_the_engine_plays() -> XrfResult {
  // `camera_effects\head_shot.anm`, which every one of the workspace trees ships: -1 to 60, which the engine reads
  // as 62 frames. Read unsigned, the start would be 4294967295 and the animation would last no time at all.
  let mut animation: AnmFile =
    AnmFile::read_from_bytes::<XRayByteOrder>(new_animation_bytes(AnmFile::CURRENT_VERSION)?)?;

  animation.frame_start = -1;
  animation.frame_end = 60;

  assert_eq!(animation.get_frame_count(), 62);
  assert_eq!(animation.get_duration_seconds(), 62.0 / 30.0);

  Ok(())
}

#[test]
fn a_range_running_backwards_spans_no_frames_rather_than_a_negative_count() -> XrfResult {
  let mut animation: AnmFile =
    AnmFile::read_from_bytes::<XRayByteOrder>(new_animation_bytes(AnmFile::CURRENT_VERSION)?)?;

  animation.frame_start = 60;
  animation.frame_end = 0;

  assert_eq!(animation.get_frame_count(), 0);
  assert_eq!(animation.get_duration_seconds(), 0.0);

  Ok(())
}

#[test]
fn a_rate_the_file_cannot_be_played_at_falls_back_to_the_format_default() -> XrfResult {
  let mut animation: AnmFile =
    AnmFile::read_from_bytes::<XRayByteOrder>(new_animation_bytes(AnmFile::CURRENT_VERSION)?)?;

  animation.fps = 0.0;

  // 480 frames at the 30 the format starts at, rather than a division with no answer.
  assert_eq!(animation.get_duration_seconds(), 16.0);

  Ok(())
}

#[test]
fn a_channel_reports_the_span_its_own_keys_cover() -> XrfResult {
  let animation: AnmFile = AnmFile::read_from_bytes::<XRayByteOrder>(new_animation_bytes(AnmFile::CURRENT_VERSION)?)?;

  assert_eq!(
    animation.get_channel(AnmChannel::PositionX).get_duration_seconds(),
    Some(8.0)
  );
  assert_eq!(
    animation.get_channel(AnmChannel::PositionZ).get_duration_seconds(),
    None
  );

  Ok(())
}

/// A motion from frame 0 to 60 at 30 frames a second, two seconds long, its x rising 3 metres a second and its heading
/// half a radian a second.
fn new_moving_motion() -> AnmFile {
  let line = |values: [f32; 2]| AnimationEnvelope {
    keys: vec![
      AnimationKey {
        value: values[0],
        time: 0.0,
        shape: AnimationShape::Linear,
        ..AnimationKey::default()
      },
      AnimationKey {
        value: values[1],
        time: 2.0,
        shape: AnimationShape::Linear,
        ..AnimationKey::default()
      },
    ],
    ..AnimationEnvelope::default()
  };
  let mut channels: [AnimationEnvelope; 6] = Default::default();

  channels[AnmChannel::PositionX.get_index()] = line([0.0, 6.0]);
  channels[AnmChannel::RotationHeading.get_index()] = line([0.0, 1.0]);

  AnmFile {
    name: String::new(),
    frame_start: 0,
    frame_end: 60,
    fps: 30.0,
    version: AnmFile::CURRENT_VERSION,
    channels,
  }
}

#[test]
fn a_looped_motion_wraps_by_whole_lengths() {
  let motion: AnmFile = new_moving_motion();
  let looped = |elapsed: Duration| motion.get_played_time(elapsed, AnimationPlayback::Looped);

  assert_eq!(looped(Duration::from_millis(500)), 0.5);
  assert_eq!(looped(Duration::from_secs(2)), 2.0);
  assert!((looped(Duration::from_millis(5500)) - 1.5).abs() < 1e-5);
}

#[test]
fn a_looped_motion_keeps_its_pace_after_a_day() {
  let motion: AnmFile = new_moving_motion();
  let looped = |elapsed: Duration| motion.get_played_time(elapsed, AnimationPlayback::Looped);
  let day: Duration = Duration::from_secs(24 * 60 * 60);
  let step: f32 = looped(day + Duration::from_millis(6)) - looped(day);

  assert!((step - 0.006).abs() < 1e-6, "{step}");
}

#[test]
fn a_motion_played_once_holds_its_last_frame() {
  // `SAnimParams::Update` sets the time to `max_t` once past it, where a looped motion wraps.
  let motion: AnmFile = new_moving_motion();
  let once = |elapsed: Duration| motion.get_played_time(elapsed, AnimationPlayback::Once);

  assert_eq!(once(Duration::from_millis(500)), 0.5);
  assert_eq!(once(Duration::from_secs(2)), 2.0);
  assert_eq!(once(Duration::from_millis(5500)), 2.0);
  assert_eq!(once(Duration::from_secs(24 * 60 * 60)), 2.0);
  assert_eq!(
    motion.evaluate(once(Duration::from_secs(9))).0,
    Vector3d::new(6.0, 0.0, 0.0)
  );
}

#[test]
fn a_motion_plays_from_its_first_frame_either_way() {
  // `SAnimParams::Set` starts at the first frame's time, -1 for `camera_effects\head_shot.anm`, and a motion of one
  // frame has no length to play over.
  let mut motion: AnmFile = new_moving_motion();

  motion.frame_start = -15;

  for playback in [AnimationPlayback::Once, AnimationPlayback::Looped] {
    assert_eq!(motion.get_played_time(Duration::ZERO, playback), -0.5);
    assert_eq!(motion.get_played_time(Duration::from_millis(500), playback), 0.0);
  }

  motion.frame_end = motion.frame_start;

  for playback in [AnimationPlayback::Once, AnimationPlayback::Looped] {
    assert_eq!(motion.get_played_time(Duration::from_secs(3), playback), -0.5);
  }
}

#[test]
fn a_motion_places_its_heading_in_y_and_its_pitch_in_x() {
  let (position, rotation) = new_moving_motion().evaluate(1.0);

  assert!((position.x - 3.0).abs() < 1e-5);
  // `_Evaluate` puts `ctRotationH` in `y` and `ctRotationP` in `x`.
  assert!((rotation.y - 0.5).abs() < 1e-5);
  assert_eq!(rotation.x, 0.0);
}
