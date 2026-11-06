//! What the water's uniform takes of its settings.

use crate::contract::feature::weather::water::render_water_mode::RenderWaterMode;
use crate::contract::feature::weather::water::render_water_settings::RenderWaterSettings;
use crate::feature::weather::water::water_uniform::WaterUniform;
use crate::tests::support::test_time::at;

#[test]
fn the_engines_water_lifts_by_its_wave_and_the_enhanced_water_stands_flat() {
  let engine: RenderWaterSettings = RenderWaterSettings::default();
  let enhanced: RenderWaterSettings = RenderWaterSettings {
    mode: RenderWaterMode::Enhanced,
    ..engine
  };

  assert_eq!(
    WaterUniform::new(&engine, 1.0, at(600.0)).wave_height,
    engine.wave_height
  );
  assert_eq!(WaterUniform::new(&enhanced, 1.0, at(600.0)).wave_height, 0.0);
}
