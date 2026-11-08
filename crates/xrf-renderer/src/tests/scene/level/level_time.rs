use crate::scene::level::level_time::LevelTime;
use crate::tests::support::test_time::{at, frame_at};

/// A view's frame steps from its last, back where the clock ran back, and not at all on its first, which stands still
/// since itself.
#[test]
fn steps_a_views_frame_from_its_last() {
  assert_eq!(frame_at(10.5, Some(10.0)).get_step(), Some(0.5));
  assert_eq!(frame_at(600.0, Some(900.0)).get_step(), Some(-300.0));
  assert_eq!(frame_at(10.0, None).get_step(), None);
  assert_eq!(frame_at(10.0, None).get_previous(), at(10.0));
  // A summed clock moves on only by a step forward.
  assert_eq!(frame_at(10.5, Some(10.0)).get_advance(), 0.5);
  assert_eq!(frame_at(600.0, Some(900.0)).get_advance(), 0.0);
  assert_eq!(frame_at(10.0, None).get_advance(), 0.0);
}

/// A phase taken within its period is the time itself until the first repeat, then what is left of it.
#[test]
fn takes_a_phase_within_its_period() {
  assert_eq!(at(600.0).get_phase(3600.0), 600.0);
  assert!((at(3700.25).get_phase(3600.0) - 100.25).abs() < 1e-4);
  assert!((at(10.0).get_turns(0.25) - 0.5).abs() < 1e-6);
  assert!((at(2.0).get_cycle(3.0, 4.0) - 2.0).abs() < 1e-6);
  // No period, or a product past `f64`, reads as nought rather than a NaN a shader would spread.
  assert_eq!(at(10.0).get_phase(0.0), 0.0);
  assert_eq!(LevelTime::wrap(f64::INFINITY, 1.0), 0.0);
  assert_eq!(LevelTime::wrap(-0.25, 1.0), 0.75);
}

/// After a day an `f32` second steps by 7.8 ms, more than a frame at 160 Hz; a phase taken from the whole time still
/// moves by the frame's own step.
#[test]
fn moves_a_frames_step_after_a_day() {
  let step: f64 = 1.0 / 160.0;
  let day: f64 = 86_400.0;
  let moved: f32 = at(day + step).get_phase(10.0) - at(day).get_phase(10.0);

  assert!((moved - step as f32).abs() < 1e-5, "{moved}");
  assert!((at(day + step).get_since(at(day)) - step as f32).abs() < 1e-6);
  // A step back is none.
  assert_eq!(at(day).get_since(at(day + step)), 0.0);
}
