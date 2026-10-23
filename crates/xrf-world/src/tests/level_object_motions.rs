use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use glam::{Mat4, Vec3};
use xrf_animation_envelope::{AnimationEnvelope, AnimationKey, AnimationShape};
use xrf_anm::AnmFile;
use xrf_renderer::RenderLevelSource;

use crate::level::effects::level_object_motions::LevelObjectMotions;
use crate::tests::fixtures::TestLevelSource;
use crate::tests::test_workers::create_workers;

/// `line.anm`: two seconds carrying its object 3 metres a second along x.
fn create_line_motion() -> AnmFile {
  let key = |time: f32, value: f32| AnimationKey {
    value,
    time,
    shape: AnimationShape::Stepped,
    ..AnimationKey::default()
  };
  let still = AnimationEnvelope::default;

  AnmFile {
    name: String::new(),
    frame_start: 0,
    frame_end: 60,
    fps: 30.0,
    version: AnmFile::CURRENT_VERSION,
    channels: vec![
      AnimationEnvelope {
        keys: vec![
          key(0.0, 0.0),
          AnimationKey {
            shape: AnimationShape::Linear,
            ..key(2.0, 6.0)
          },
        ],
        ..AnimationEnvelope::default()
      },
      still(),
      still(),
      still(),
      still(),
      still(),
    ],
  }
}

/// A level with nothing but `line.anm`.
fn create_source() -> Arc<dyn RenderLevelSource> {
  Arc::new(TestLevelSource {
    motions: HashMap::from([(String::from("line.anm"), create_line_motion())]),
    ..TestLevelSource::default()
  })
}

/// Asks for a motion until its loader has answered, a few seconds at most.
fn read(motions: &mut LevelObjectMotions, name: &str) -> Option<(Mat4, Vec3)> {
  let started: Instant = Instant::now();

  loop {
    if let Some(pose) = motions.get_pose(name) {
      return Some(pose);
    }

    if started.elapsed() > Duration::from_secs(5) {
      return None;
    }

    std::thread::sleep(Duration::from_millis(5));
  }
}

// At 1.5 s the object stands 4.5 m along x, and since the frame at 1 s it moved 1.5 m in half a second.
#[test]
fn moves_an_object_along_its_motion_with_its_velocity() {
  let source: Arc<dyn RenderLevelSource> = create_source();
  let mut motions: LevelObjectMotions = LevelObjectMotions::new(&source, &create_workers());

  motions.prepare(Duration::from_millis(1000));
  motions.prepare(Duration::from_millis(1500));

  let (transform, velocity) = read(&mut motions, "line.anm").expect("the motion to be read");

  assert!(transform.w_axis.truncate().abs_diff_eq(Vec3::new(4.5, 0.0, 0.0), 1e-4));
  assert!(velocity.abs_diff_eq(Vec3::new(3.0, 0.0, 0.0), 1e-3));
}

// From 1.9 s to 2.1 s the two-second motion wraps: it still moves at its 3 m a second, not back across the seam.
#[test]
fn keeps_an_object_s_velocity_across_its_loop_s_seam() {
  let source: Arc<dyn RenderLevelSource> = create_source();
  let mut motions: LevelObjectMotions = LevelObjectMotions::new(&source, &create_workers());

  motions.prepare(Duration::from_millis(1900));
  motions.prepare(Duration::from_millis(2100));

  let (_, velocity) = read(&mut motions, "line.anm").expect("the motion to be read");

  assert!(velocity.abs_diff_eq(Vec3::new(3.0, 0.0, 0.0), 1e-3), "{velocity}");
}

/// A next source of the same level, whose `line.anm` stands its object still at the start of its line.
fn create_still_source() -> Arc<dyn RenderLevelSource> {
  let mut still: AnmFile = create_line_motion();

  still.channels[0].keys.truncate(1);

  Arc::new(TestLevelSource {
    motions: HashMap::from([(String::from("line.anm"), still)]),
    ..TestLevelSource::default()
  })
}

#[test]
fn reads_its_motions_again_from_a_next_source_only_when_told() {
  let source: Arc<dyn RenderLevelSource> = create_source();
  let mut motions: LevelObjectMotions = LevelObjectMotions::new(&source, &create_workers());

  motions.prepare(Duration::from_millis(1000));
  read(&mut motions, "line.anm").expect("the motion to be read");

  motions.follow(&create_still_source(), false);

  let (kept, _) = read(&mut motions, "line.anm").expect("the motion read before");

  assert!(kept.w_axis.truncate().abs_diff_eq(Vec3::new(3.0, 0.0, 0.0), 1e-4));

  motions.follow(&create_still_source(), true);

  let (reread, _) = read(&mut motions, "line.anm").expect("the motion read again");

  assert!(reread.w_axis.truncate().abs_diff_eq(Vec3::ZERO, 1e-4), "{reread}");
}

#[test]
fn leaves_an_object_standing_without_its_motion() {
  let source: Arc<dyn RenderLevelSource> = create_source();
  let mut motions: LevelObjectMotions = LevelObjectMotions::new(&source, &create_workers());

  motions.prepare(Duration::from_millis(1000));

  let started: Instant = Instant::now();

  while motions.get_pose("gone.anm").is_none()
    && motions.is_pending("gone.anm")
    && started.elapsed() < Duration::from_secs(5)
  {
    std::thread::sleep(Duration::from_millis(5));
  }

  assert!(!motions.is_pending("gone.anm"));
  assert!(motions.get_pose("gone.anm").is_none());
}
