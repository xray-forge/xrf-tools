//! Holds what a baked motion reports about itself: the frames it poses, and how long playing them takes.

use xrf_db::{OgfBone, OgfBoneIkData, OgfMotion, OgfMotionDefinition, OgfPart, SAMPLE_FPS};
use xrf_error::XrfResult;

use crate::pack::tests::fixtures::{bind, bones, vector};
use crate::pack::visual_motion::{VisualMotionPose, bake_motion};

/// The measured case behind the reported duration: `s98b_reload` of Anomaly's `ceasar_hand_am98b_hud_animation.omf`.
const RELOAD_FRAMES: u32 = 269;
const RELOAD_SPEED: f32 = 1.2;

/// A motion holding its one bone still: a rotation key and an offset, which is what the payload stores for it.
fn held_motion(count: u32) -> OgfMotion {
  let mut remaining: Vec<u8> = Vec::new();

  remaining.extend([0_u8; 8]);
  remaining.extend([0_u8; 12]);

  OgfMotion {
    label: String::from("stale_label"),
    count,
    // `FL_R_KEY_ABSENT`, so the bone stores one key rather than one per frame.
    flags: 1 << 1,
    remaining,
  }
}

/// A motion animating its one bone over no frames: two checksums, then the scale and offset of an empty key stream.
///
/// Structurally complete, so the payload is consumed exactly - the zero count is the only thing wrong with it.
fn zero_frame_keyed_motion() -> OgfMotion {
  OgfMotion {
    label: String::from("stale_label"),
    count: 0,
    // `FL_T_KEY_PRESENT | FL_T_KEY_16_IS_BIT`, so both streams store one key a frame rather than one in total.
    flags: (1 << 0) | (1 << 2),
    remaining: vec![0; 4 + 4 + 12 + 12],
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

fn try_bake(motion: &OgfMotion, speed: f32) -> XrfResult<VisualMotionPose> {
  let skeleton: Vec<OgfBone> = bones(&[("wpn_body", "")]).bones;
  let binds: Vec<OgfBoneIkData> = vec![bind(vector(0.0, 0.0, 0.0), vector(0.0, 0.0, 0.0))];
  let parts: Vec<OgfPart> = vec![OgfPart {
    name: String::from("default"),
    bones: vec![(String::from("wpn_body"), 0)],
  }];

  bake_motion(&skeleton, &binds, &parts, &definition("s98b_reload", speed), motion)
}

fn bake(count: u32, speed: f32) -> VisualMotionPose {
  try_bake(&held_motion(count), speed).expect("expect a one bone motion to bake")
}

#[test]
fn reports_the_frame_span_when_the_motion_plays_at_the_sample_rate() {
  let posed: VisualMotionPose = bake(RELOAD_FRAMES, 1.0);

  assert_eq!(posed.description.frame_count, RELOAD_FRAMES);
  assert_eq!(posed.description.speed, 1.0);
  assert_eq!(posed.description.duration, RELOAD_FRAMES as f32 / SAMPLE_FPS);
}

#[test]
fn reports_a_faster_motion_as_taking_less_time_than_its_frames_span() {
  // The engine divides the frame span by the speed, so 269 frames at 1.2 take 7.47 seconds rather than 8.97.
  let posed: VisualMotionPose = bake(RELOAD_FRAMES, RELOAD_SPEED);

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
  let posed: VisualMotionPose = bake(RELOAD_FRAMES, 0.0);

  assert_eq!(posed.description.speed, 0.0);
  assert_eq!(posed.description.duration, RELOAD_FRAMES as f32 / SAMPLE_FPS);
}

#[test]
fn reports_an_empty_motion_as_the_one_frame_it_poses() {
  // The bake clamps a zero frame count to the single frame it actually composes, and the duration follows that rather
  // than the count the file declares.
  let posed: VisualMotionPose = bake(0, RELOAD_SPEED);

  assert_eq!(posed.description.frame_count, 1);
  assert_eq!(posed.description.duration, 1.0 / (SAMPLE_FPS * RELOAD_SPEED));
}

#[test]
fn refuses_a_keyed_motion_that_declares_no_frames() {
  // The bake poses at least one frame whatever the count says, so an empty keyed stream would be indexed at frame
  // zero. The decoder refuses the motion instead of handing back streams no frame can be sampled from.
  let error: String = try_bake(&zero_frame_keyed_motion(), RELOAD_SPEED)
    .expect_err("expect a zero frame keyed motion to be refused")
    .to_string();

  assert!(
    error.contains("declares zero frames but a bone carries rotation keys"),
    "Unexpected error: {error}"
  );
}
