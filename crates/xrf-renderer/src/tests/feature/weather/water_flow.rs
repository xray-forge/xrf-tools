//! How far the enhanced water's maps scroll, summed frame by frame, and where each map read stands.

use crate::contract::feature::weather::water::render_enhanced_water_settings::RenderEnhancedWaterSettings;
use crate::feature::weather::water::enhanced::enhanced_water_scroll::EnhancedWaterScroll;
use crate::feature::weather::water::enhanced::water_flow::WaterFlow;
use crate::lighting::wind::render_wind::RenderWind;
use crate::tests::support::test_time::{at, frame_at};

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

  flow.advance(frame_at(1000.0, None), &settings, wind(0.0));
  flow.advance(frame_at(1100.0, Some(1000.0)), &settings, wind(0.0));

  let calm: WaterFlow = flow;

  // A strong wind a frame later moves the maps by that frame's seconds alone, at its own pace.
  flow.advance(frame_at(1100.1, Some(1100.0)), &settings, wind(1000.0));

  assert!((flow.waves - calm.waves - 0.1 * 0.97).abs() < 1e-6);
  assert!(((flow.gusts[0] - calm.gusts[0]).hypot(flow.gusts[1] - calm.gusts[1]) - 0.1).abs() < 1e-6);
  // Before it, the calm drifted at its floor times the calm flow, for the hundred seconds it was drawn.
  assert!((calm.waves - 100.0 * 0.45 * f64::from(settings.calm_flow)).abs() < 1e-6);
  assert_eq!(calm.gusts, [0.0; 2]);
}

#[test]
fn the_flow_scales_every_scroll_and_none_stills_them() {
  let still: RenderEnhancedWaterSettings = RenderEnhancedWaterSettings {
    flow: 0.0,
    ..RenderEnhancedWaterSettings::default()
  };
  let mut flow: WaterFlow = WaterFlow::default();

  flow.advance(frame_at(0.0, None), &still, wind(500.0));
  flow.advance(frame_at(60.0, Some(0.0)), &still, wind(500.0));

  assert_eq!(
    [flow.seconds, flow.waves, flow.heights, flow.gusts[0], flow.gusts[1]],
    [0.0; 5]
  );
}

/// A frame's step after a day of flowing moves the maps as far as one in the first minute: the sums keep it, and each
/// read is taken within its map's repeat before it becomes `f32`.
#[test]
fn a_frame_moves_the_maps_alike_after_a_day() {
  let settings: RenderEnhancedWaterSettings = RenderEnhancedWaterSettings::default();
  let step = |start: f64| {
    let mut flow: WaterFlow = WaterFlow::default();

    flow.advance(frame_at(start, None), &settings, wind(400.0));
    flow.advance(frame_at(start + 60.0, Some(start)), &settings, wind(400.0));

    let before: EnhancedWaterScroll = EnhancedWaterScroll::new(&flow, at(start + 60.0), 0.5, false);

    flow.advance(
      frame_at(start + 60.0 + 1.0 / 160.0, Some(start + 60.0)),
      &settings,
      wind(400.0),
    );

    let after: EnhancedWaterScroll = EnhancedWaterScroll::new(&flow, at(start + 60.0 + 1.0 / 160.0), 0.5, false);
    let moved = |from: f32, to: f32| {
      let change: f32 = (to - from).rem_euclid(1.0);

      change.min(1.0 - change)
    };

    [
      moved(before.heights.x, after.heights.x),
      moved(before.wave_layers[1].z, after.wave_layers[1].z),
      moved(before.map_offsets[1].z, after.map_offsets[1].z),
      moved(before.ripple_clocks.y, after.ripple_clocks.y),
    ]
  };
  let (minute, day): ([f32; 4], [f32; 4]) = (step(0.0), step(86_400.0));

  for (first, later) in minute.into_iter().zip(day) {
    assert!(first > 0.0, "{minute:?}");
    assert!((first - later).abs() < first * 1e-3, "{minute:?} against {day:?}");
  }
}

/// Each read stands where the sums put it, taken within one repeat: the waves' scroll added to each layer as the layer
/// turns and tiles its coordinates, none of it where the parallax marches them.
#[test]
fn each_read_stands_where_the_sums_put_it() {
  let mut flow: WaterFlow = WaterFlow::default();

  (flow.seconds, flow.waves, flow.heights, flow.gusts) = (12.5, 7.25, 3.5, [0.75, -1.5]);

  let scroll: EnhancedWaterScroll = EnhancedWaterScroll::new(&flow, at(10.0), 0.5, false);
  let marched: EnhancedWaterScroll = EnhancedWaterScroll::new(&flow, at(10.0), 0.5, true);
  let wrap = |value: f64| value.rem_euclid(1.0) as f32;
  let (across, down): (f64, f64) = (7.25 * 0.065, 7.25 * 0.445);
  let (cos, sin, tile): (f64, f64, f64) = (0.484810, 0.874620, f64::from(EnhancedWaterScroll::WAVE_TILES[1]));

  assert!((scroll.heights.x - wrap(3.5 * 0.065)).abs() < 1e-6);
  assert!((scroll.heights.w - wrap(3.5 * 0.241)).abs() < 1e-6);
  assert!((scroll.wave_layers[0].z - wrap((cos * across - sin * down) * tile + 7.25 * 0.3 / tile.sqrt())).abs() < 1e-5);
  assert!((scroll.wave_layers[0].w - wrap((sin * across + cos * down) * tile)).abs() < 1e-5);
  assert!((marched.wave_layers[0].z - wrap(7.25 * 0.3 / tile.sqrt())).abs() < 1e-5);
  assert_eq!(marched.wave_layers[0].w, 0.0);
  assert!((scroll.map_offsets[1].z - wrap((across + 0.75) * 0.1)).abs() < 1e-6);
  assert!((marched.map_offsets[1].w - wrap(-1.5 * 0.1)).abs() < 1e-6);
  assert!((scroll.flow_scrolls.y - wrap(12.5 * 0.07)).abs() < 1e-6);
  // The ripples' clocks at the rain's pace, 1.6 times its density held to 0.65 and one.
  assert!((scroll.ripple_clocks.x - wrap(10.0 * 1.05 * 0.8)).abs() < 1e-6);
}
