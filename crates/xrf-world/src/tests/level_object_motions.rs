use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use glam::{Mat4, Vec3};
use xrf_animation_envelope::{AnimationEnvelope, AnimationKey};
use xrf_anm::AnmFile;
use xrf_renderer::RenderLevelSource;

use crate::level::level_object_motions::LevelObjectMotions;
use crate::tests::fixtures::TestLevelSource;
use crate::tests::test_workers::create_workers;

/// `line.anm`: two seconds carrying its object 3 metres a second along x.
fn create_line_motion() -> AnmFile {
  let key = |time: f32, value: f32| AnimationKey {
    value,
    time,
    shape: 4,
    interpolation: None,
  };
  let still = || AnimationEnvelope {
    behavior: (1, 1),
    keys: Vec::new(),
  };

  AnmFile {
    name: String::new(),
    frame_start: 0,
    frame_end: 60,
    fps: 30.0,
    version: AnmFile::CURRENT_VERSION,
    channels: vec![
      AnimationEnvelope {
        behavior: (1, 1),
        keys: vec![
          key(0.0, 0.0),
          AnimationKey {
            shape: 3,
            ..key(2.0, 6.0)
          },
        ],
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
