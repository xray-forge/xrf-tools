use crate::sky::hemisphere::Hemisphere;
use crate::sky::hemisphere_light::HemisphereLight;

#[test]
fn spreads_the_skys_energy_over_each_qualitys_directions_downwards() {
  for (quality, count) in [(1, 26), (2, 91), (3, 196)] {
    let lights: Vec<HemisphereLight> = Hemisphere::get_lights(quality, 2.0);

    assert_eq!(lights.len(), count);
    assert!((lights.iter().map(|light| light.energy).sum::<f32>() - 2.0).abs() < 1e-4);
    assert!(
      lights
        .iter()
        .all(|light| light.direction.y <= 0.0 && (light.direction.length() - 1.0).abs() < 1e-5)
    );
  }
}

#[test]
fn gives_no_light_for_a_quality_without_a_table() {
  assert!(Hemisphere::get_lights(0, 2.0).is_empty());
  assert!(Hemisphere::get_lights(4, 2.0).is_empty());
}
