use xrf_error::XrfResult;
use xrf_math::Vector3d;

use crate::sky::sky::Sky;
use crate::sky::sky_light::SkyLight;

#[test]
fn spreads_the_skys_energy_over_each_qualitys_directions_downwards() {
  for (quality, count) in [(1, 26), (2, 91), (3, 196)] {
    let lights: Vec<SkyLight> = Sky::get_hemisphere(quality);

    assert_eq!(lights.len(), count);
    assert!((lights.iter().map(|light| light.energy).sum::<f32>() - Sky::HEMI_ENERGY).abs() < 1e-4);
    assert!(
      lights
        .iter()
        .all(|light| light.direction.y <= 0.0 && (light.direction.length() - 1.0).abs() < 1e-5)
    );
  }
}

#[test]
fn lights_the_lowest_quality_sky_straight_down_and_another_not_at_all() {
  assert_eq!(
    Sky::get_hemisphere(0),
    vec![SkyLight {
      direction: Vector3d::new(0.0, -1.0, 0.0),
      energy: 1.0,
    }]
  );
  assert!(Sky::get_hemisphere(4).is_empty());
}

#[test]
fn generates_the_sun_as_the_shipped_editor_does() -> XrfResult {
  let (pitch, heading): (f32, f32) = (-0.5, 1.7);
  let step: f32 = 3.0_f32.to_radians() / 7.0;
  let (pitch_from, heading_from) = (pitch - 3.0_f32.to_radians() / 2.0, heading - 3.0_f32.to_radians() / 2.0);
  let lights: Vec<SkyLight> = Sky::get_sun(3, pitch, heading)?;
  let get_heading = |light: &SkyLight| Sky::get_pitch_heading(&light.direction).1;

  assert_eq!(lights.len(), 49);
  assert!((lights[0].energy - 1.0 / 49.0).abs() < 1e-7);
  // Each sample from the one before: the second heading is the first's taken through the step once more.
  assert!((get_heading(&lights[0]) - heading_from).abs() < 1e-5);
  assert!((get_heading(&lights[1]) - (heading_from + heading_from * step)).abs() < 1e-5);
  assert!((Sky::get_pitch_heading(&lights[0].direction).0 - pitch_from).abs() < 1e-5);
  assert_eq!(Sky::get_sun(1, pitch, heading)?.len(), 9);
  assert!(Sky::get_sun(4, pitch, heading).is_err());

  Ok(())
}

#[test]
fn reads_back_the_pitch_and_heading_a_direction_is_built_from() {
  let (pitch, heading): (f32, f32) = Sky::get_pitch_heading(&Sky::get_direction(2.1, -0.7));

  assert!((pitch + 0.7).abs() < 1e-5);
  assert!((heading - 2.1).abs() < 1e-5);

  // A heading west of north comes back a turn on, as the level options hold it.
  let (_, heading): (f32, f32) = Sky::get_pitch_heading(&Sky::get_direction(-1.2, -0.4));

  assert!((heading - (std::f32::consts::TAU - 1.2)).abs() < 1e-5);
}
