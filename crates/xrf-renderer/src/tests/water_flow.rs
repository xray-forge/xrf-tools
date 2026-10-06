//! How far the enhanced water's maps scroll, summed frame by frame.

use crate::contract::render_enhanced_water_settings::RenderEnhancedWaterSettings;
use crate::lighting::render_wind::RenderWind;
use crate::scene::level::water_flow::WaterFlow;

fn wind(velocity: f32) -> RenderWind {
  RenderWind {
    direction: 0.0,
    velocity,
  }
}

#[test]
fn a_change_of_wind_changes_how_fast_the_water_moves_not_where_it_stands() {
  let settings: RenderEnhancedWaterSettings = RenderEnhancedWaterSettings::default();
  let mut flow: WaterFlow = WaterFlow::default();

  flow.advance(1000.0, &settings, wind(0.0));
  flow.advance(1100.0, &settings, wind(0.0));

  let calm: WaterFlow = flow;

  // A strong wind a frame later moves the maps by that frame's seconds alone, at its own pace.
  flow.advance(1100.1, &settings, wind(1000.0));

  assert!((flow.waves - calm.waves - 0.1 * 0.97).abs() < 1e-3);
  assert!((flow.gusts[0] - calm.gusts[0]).hypot(flow.gusts[1] - calm.gusts[1]) - 0.1 < 1e-3);
  // Before it, the calm drifted at its floor times the calm flow, for the hundred seconds it was drawn.
  assert!((calm.waves - 100.0 * 0.45 * settings.calm_flow).abs() < 1e-2);
  assert_eq!(calm.gusts, [0.0; 2]);
}

#[test]
fn the_flow_scales_every_scroll_and_none_stills_them() {
  let still: RenderEnhancedWaterSettings = RenderEnhancedWaterSettings {
    flow: 0.0,
    ..RenderEnhancedWaterSettings::default()
  };
  let mut flow: WaterFlow = WaterFlow::default();

  flow.advance(0.0, &still, wind(500.0));
  flow.advance(60.0, &still, wind(500.0));

  assert_eq!(
    [flow.seconds, flow.waves, flow.heights, flow.gusts[0], flow.gusts[1]],
    [0.0; 5]
  );
}
