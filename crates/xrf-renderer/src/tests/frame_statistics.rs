use std::time::{Duration, Instant};

use crate::contract::render_rect::RenderRect;
use crate::frame::frame_phases::FramePhases;
use crate::frame::frame_statistics::FrameStatistics;

#[test]
fn reports_only_once_due_and_drawn() {
  let start: Instant = Instant::now();
  let mut statistics: FrameStatistics = FrameStatistics::new(start);

  assert!(statistics.take(start + Duration::from_secs(1)).is_none());

  let phases = |prepare: u64, present: u64| FramePhases {
    prepare: Duration::from_millis(prepare),
    present: Duration::from_millis(present),
    ..FramePhases::default()
  };

  statistics.record(Duration::from_millis(10), Duration::from_millis(1), &phases(2, 4));
  statistics.record(Duration::from_millis(30), Duration::from_millis(3), &phases(4, 8));

  assert!(statistics.take(start + Duration::from_millis(100)).is_none());

  let summary = statistics.take(start + Duration::from_millis(500)).unwrap();

  assert!((summary.frames_per_second - 4.0).abs() < 1e-3);
  assert!((summary.frame_time - 20.0).abs() < 1e-3);
  assert!((summary.frame_time_max - 30.0).abs() < 1e-3);
  assert!((summary.cpu_time - 2.0).abs() < 1e-3);
  // Each phase is averaged over the frames, apart from the others.
  assert!((summary.phases.prepare - 3.0).abs() < 1e-3);
  assert!((summary.phases.present - 6.0).abs() < 1e-3);
  assert_eq!(summary.phases.acquire, 0.0);
}

#[test]
fn clips_a_rect_to_its_surface() {
  let rect: RenderRect = RenderRect {
    x: -10,
    y: 20,
    width: 100,
    height: 50,
  };

  assert_eq!(
    rect.clip(60, 60),
    Some(RenderRect {
      x: 0,
      y: 20,
      width: 60,
      height: 40
    })
  );
  assert_eq!(rect.clip(10, 10), None);
}
