//! How lit the camera stands comes with the level, not with its particles.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_geometry::BvhTracer;
use xrf_renderer::{RenderSceneFrame, RenderSceneId, RenderViewportId, RenderWorld};
use xrf_visual::HemiEstimator;

use crate::tests::fixtures::{SilentSink, TestLevelSource};
use crate::tests::test_workers::create_workers;
use crate::world::World;

/// Longer than a test level's reads and a first estimate take on the test workers.
const WAIT: Duration = Duration::from_secs(10);

/// `hemi_value` under open sky: every one of the 26 samples open, times 0.08.
const OPEN_SKY: f32 = 0.08;

/// What a level's reads are given to land before a test says none brought an estimator.
const QUIET: Duration = Duration::from_millis(300);

/// The camera's hemi the level's frames report, once one meets `until` or `wait` runs out.
fn advance_until(source: TestLevelSource, wait: Duration, until: impl Fn(Option<f32>) -> bool) -> Option<f32> {
  let mut world: World = World::new(create_workers());
  let viewport: RenderViewportId = RenderViewportId(1);

  world.attach(viewport, Box::new(SilentSink));
  world
    .show_level(viewport, Some(Arc::new(source)))
    .expect("the level shown");

  let started: Instant = Instant::now();
  let mut hemi: Option<f32> = None;

  while started.elapsed() < wait {
    let frames: HashMap<RenderSceneId, RenderSceneFrame> = world.advance_scenes(Instant::now(), HashMap::new());

    hemi = frames.values().find_map(|frame| frame.camera_hemi);

    if until(hemi) {
      break;
    }

    std::thread::sleep(Duration::from_millis(2));
  }

  hemi
}

// The test level reads no particles, as a level whose `particles.xr` does not read: the fog's indoor share and the
// ambient effects' indoor test still have the camera's hemi.
#[test]
fn estimates_the_camera_hemi_of_a_level_whose_particles_do_not_read() {
  let open: BvhTracer = BvhTracer::new(&[], &[]);
  let hemi: Option<f32> = advance_until(
    TestLevelSource {
      hemi: Some(Arc::new(HemiEstimator::new(Arc::new(open), &[]))),
      ..TestLevelSource::default()
    },
    WAIT,
    |hemi| hemi.is_some_and(|hemi| (hemi - OPEN_SKY).abs() < 1e-4),
  );

  assert!(
    hemi.is_some_and(|hemi| (hemi - OPEN_SKY).abs() < 1e-4),
    "the open sky estimated: {hemi:?}"
  );
}

#[test]
fn reports_no_camera_hemi_for_a_level_without_an_estimator() {
  let hemi: Option<f32> = advance_until(TestLevelSource::default(), QUIET, |hemi| hemi.is_some());

  assert_eq!(hemi, None);
}
