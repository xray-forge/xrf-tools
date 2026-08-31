//! Holds what a baked motion reports about itself: the frames it poses, and how long playing them takes.

use xrf_db::{OgfBone, OgfBoneIkData, OgfMotion, OgfMotionDefinition, OgfPart, SAMPLE_FPS};
use xrf_error::XrfResult;

use crate::pack::tests::fixtures::{bind, bones, vector};
use crate::pack::visual_motion::{FLOATS_PER_BONE, VisualMotionPose, bake_motion};

/// The measured case behind the reported duration: `s98b_reload` of Anomaly's `ceasar_hand_am98b_hud_animation.omf`.
const RELOAD_FRAMES: u32 = 269;
const RELOAD_SPEED: f32 = 1.2;

/// One bone's held run: a rotation key and an offset, with the flags that say to read only those.
///
/// Twenty bytes whatever the motion's count says, which is the payload independence the frame count is derived around.
fn held_run() -> (u8, Vec<u8>) {
  // `FL_R_KEY_ABSENT`, so the bone stores one key rather than one per frame, and no checksum precedes it.
  (1 << 1, vec![0; 8 + 12])
}

/// One bone's keyed run over `count` frames: a checksum and a stream each, then the scale and offset to dequantise by.
fn keyed_run(count: u32) -> (u8, Vec<u8>) {
  let count: usize = count as usize;

  // `FL_T_KEY_PRESENT | FL_T_KEY_16_IS_BIT`, so both streams store one key a frame rather than one in total.
  ((1 << 0) | (1 << 2), vec![0; 4 + count * 8 + 4 + count * 6 + 12 + 12])
}

/// Assembles bone runs into a motion, moving the first bone's flags where the chunk reader takes them.
fn motion(count: u32, runs: &[(u8, Vec<u8>)]) -> OgfMotion {
  let mut remaining: Vec<u8> = Vec::new();
  let mut first: u8 = 0;

  for (index, (flags, bytes)) in runs.iter().enumerate() {
    if index == 0 {
      first = *flags;
    } else {
      remaining.push(*flags);
    }

    remaining.extend(bytes);
  }

  OgfMotion {
    label: String::from("stale_label"),
    count,
    flags: first,
    remaining,
  }
}

fn definition(name: &str, speed: f32) -> OgfMotionDefinition {
  OgfMotionDefinition {
    name: String::from(name),
    flags: 0,
    bone_or_part: 0,
    motion: 0,
    speed,
    power: 1.0,
    accrue: 2.0,
    falloff: 2.0,
    marks: Vec::new(),
  }
}

/// Poses a motion onto a flat skeleton of the named bones, one partition naming them in order.
fn try_bake(motion: &OgfMotion, names: &[&str], speed: f32) -> XrfResult<VisualMotionPose> {
  let skeleton: Vec<OgfBone> = bones(&names.iter().map(|it| (*it, "")).collect::<Vec<(&str, &str)>>()).bones;
  let binds: Vec<OgfBoneIkData> = names
    .iter()
    .map(|_| bind(vector(0.0, 0.0, 0.0), vector(0.0, 0.0, 0.0)))
    .collect();
  let parts: Vec<OgfPart> = vec![OgfPart {
    name: String::from("default"),
    bones: names
      .iter()
      .enumerate()
      .map(|(index, name)| (String::from(*name), index as u32))
      .collect(),
  }];

  bake_motion(&skeleton, &binds, &parts, &definition("s98b_reload", speed), motion)
}

fn bake_keyed(count: u32, speed: f32) -> VisualMotionPose {
  try_bake(&motion(count, &[keyed_run(count)]), &["wpn_body"], speed).expect("expect a one bone motion to bake")
}

fn bake_held(count: u32, speed: f32) -> VisualMotionPose {
  try_bake(&motion(count, &[held_run()]), &["wpn_body"], speed).expect("expect a one bone motion to bake")
}

#[test]
fn reports_the_frame_span_when_the_motion_plays_at_the_sample_rate() {
  let posed: VisualMotionPose = bake_keyed(RELOAD_FRAMES, 1.0);

  assert_eq!(posed.description.frame_count, RELOAD_FRAMES);
  assert_eq!(posed.description.speed, 1.0);
  assert_eq!(posed.description.duration, RELOAD_FRAMES as f32 / SAMPLE_FPS);
}

#[test]
fn reports_a_faster_motion_as_taking_less_time_than_its_frames_span() {
  // The engine divides the frame span by the speed, so 269 frames at 1.2 take 7.47 seconds rather than 8.97.
  let posed: VisualMotionPose = bake_keyed(RELOAD_FRAMES, RELOAD_SPEED);

  assert_eq!(
    posed.description.frame_count, RELOAD_FRAMES,
    "expect the frames to be untouched"
  );
  assert_eq!(posed.description.speed, RELOAD_SPEED);
  assert_eq!(
    posed.description.duration,
    RELOAD_FRAMES as f32 / (SAMPLE_FPS * RELOAD_SPEED)
  );
  assert!(
    (posed.description.duration - 7.472).abs() < 0.001,
    "unexpected duration: {}",
    posed.description.duration
  );
}

#[test]
fn reports_a_speed_that_cannot_scale_playback_as_the_frame_span() {
  // A stored zero would divide the span by nothing; the bake still reports the speed the file carries, so the two
  // together say the duration is a fallback rather than a measurement.
  let posed: VisualMotionPose = bake_keyed(RELOAD_FRAMES, 0.0);

  assert_eq!(posed.description.speed, 0.0);
  assert_eq!(posed.description.duration, RELOAD_FRAMES as f32 / SAMPLE_FPS);
}

#[test]
fn reports_an_empty_motion_as_the_one_frame_it_poses() {
  // The bake reports the single frame it composes, and the duration follows that rather than the count the file
  // declares.
  let posed: VisualMotionPose = bake_held(0, RELOAD_SPEED);

  assert_eq!(posed.description.frame_count, 1);
  assert_eq!(posed.description.duration, 1.0 / (SAMPLE_FPS * RELOAD_SPEED));
}

#[test]
fn refuses_a_keyed_motion_that_declares_no_frames() {
  // The bake poses at least one frame whatever the count says, so an empty keyed stream would be indexed at frame
  // zero. The decoder refuses the motion instead of handing back streams no frame can be sampled from.
  let error: String = try_bake(&motion(0, &[keyed_run(0)]), &["wpn_body"], RELOAD_SPEED)
    .expect_err("expect a zero frame keyed motion to be refused")
    .to_string();

  assert!(
    error.contains("declares zero frames but a bone carries rotation keys"),
    "Unexpected error: {error}"
  );
}

#[test]
fn poses_one_frame_for_a_motion_of_nothing_but_held_bones() {
  // A held run costs twenty bytes however many frames the motion declares, so the payload contradicts no count at all
  // and the decode balances. Reserving by the declared count would ask for four billion frames of a pose that never
  // changes; the streams say the motion has one.
  let posed: VisualMotionPose = bake_held(u32::MAX, RELOAD_SPEED);

  assert_eq!(posed.description.frame_count, 1);
  assert_eq!(posed.description.duration, 1.0 / (SAMPLE_FPS * RELOAD_SPEED));
  assert_eq!(
    posed.transforms.len(),
    FLOATS_PER_BONE,
    "expect one bone's transform, once"
  );
}

#[test]
fn takes_the_frame_count_from_the_longest_stream_any_bone_carries() {
  // The held bone leads, so taking the first run's length - or the shortest - would collapse a motion that does
  // animate. One keyed bone is enough to make every frame distinct.
  let posed: VisualMotionPose = try_bake(
    &motion(RELOAD_FRAMES, &[held_run(), keyed_run(RELOAD_FRAMES)]),
    &["wpn_body", "wpn_shell"],
    RELOAD_SPEED,
  )
  .expect("expect a two bone motion to bake");

  assert_eq!(posed.description.frame_count, RELOAD_FRAMES);
  assert_eq!(posed.description.bone_count, 2);
  assert_eq!(
    posed.transforms.len(),
    RELOAD_FRAMES as usize * 2 * FLOATS_PER_BONE,
    "expect the buffer to hold every baked frame"
  );
}
