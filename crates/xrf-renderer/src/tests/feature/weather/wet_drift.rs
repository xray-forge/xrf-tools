//! How far the gusts and the calm waves move across the puddles, summed frame by frame.

use glam::{Vec2, Vec4};

use crate::feature::weather::wet::wet_drift::WetDrift;
use crate::feature::weather::wet::wet_uniform::WetUniform;
use crate::tests::support::test_time::frame_at;

/// The drift's gusts in metres and its waves in repeats, wrapped as the uniform holds them.
fn drifted(drift: &WetDrift) -> Vec4 {
  drift.get()
}

/// A wind turning late in a level moves the puddles' gusts by each frame's seconds at that frame's gust, never by the
/// level's whole time at the new gust: the drift stays where it stood and goes on the new way.
#[test]
fn a_turning_wind_changes_where_the_gusts_go_not_where_they_stand() {
  let mut drift: WetDrift = WetDrift::default();
  // A metre a second along `z`, then along `x`: each within the gusts' repeat from where they start.
  let first: Vec2 = WetUniform::get_gust(std::f32::consts::PI, 100.0);
  let turned: Vec2 = WetUniform::get_gust(-std::f32::consts::FRAC_PI_2, 100.0);

  drift.advance(frame_at(36_000.0, None), first, 0.0);
  drift.advance(frame_at(36_010.0, Some(36_000.0)), first, 0.0);

  let before: Vec4 = drifted(&drift);

  drift.advance(frame_at(36_010.1, Some(36_010.0)), turned, 0.0);

  let after: Vec4 = drifted(&drift);
  let moved: Vec2 = Vec2::new(after.x - before.x, after.y - before.y);

  assert!(
    (Vec2::new(before.x, before.y) - first * 10.0).length() < 1e-4,
    "{before}"
  );
  assert!(
    (moved - turned * 0.1).length() < 1e-4,
    "{moved} against {}",
    turned * 0.1
  );
}

/// A rain thickening scrolls the calm waves faster from then on, never further back in time: a tenth of a second at the
/// heavier rain's pace, whatever the hour.
#[test]
fn a_thickening_rain_scrolls_the_waves_faster_from_then_on() {
  let mut drift: WetDrift = WetDrift::default();

  drift.advance(frame_at(36_000.0, None), Vec2::ZERO, 0.2);
  drift.advance(frame_at(36_100.0, Some(36_000.0)), Vec2::ZERO, 0.2);

  let light: f32 = drifted(&drift).z;

  drift.advance(frame_at(36_100.1, Some(36_100.0)), Vec2::ZERO, 1.0);

  // A hundred seconds at the calm pace and a light rain's share, within the waves' repeat; then a tenth at the
  // heaviest.
  assert!((light - (100.0 * (0.01 + 0.3 * 0.008_f64)).rem_euclid(1.0) as f32).abs() < 1e-5);
  assert!((drifted(&drift).z - light - 0.1 * (0.01 + 0.008)).abs() < 1e-6);
}
