use xrf_level::LevelPsStaticPlacement;
use xrf_math::Matrix4x4;
use xrf_renderer::RenderParticleSource;

use crate::parts::LevelParticlesPart;

#[test]
fn lifts_a_planted_system_a_centimetre() {
  let mut values: [f32; 16] = [0.0; 16];

  values[0] = 1.0;
  values[5] = 1.0;
  values[10] = 1.0;
  values[12] = 4.0;
  values[13] = 2.0;
  values[15] = 1.0;

  let placement = LevelParticlesPart::place_planted(&LevelPsStaticPlacement {
    game_types: Some(1),
    effect: String::from("zones\\zone_thermal_idle"),
    transform: Matrix4x4 { values },
  });

  assert_eq!(placement.transform[12], 4.0);
  assert_eq!(placement.transform[13], 2.0 + 0.01);
  assert_eq!(
    placement.source,
    RenderParticleSource::Static {
      name: String::from("zones\\zone_thermal_idle")
    }
  );
}
