use crate::lighting::grass_wind::GrassWind;
use crate::scene::level::grass_build_size::GrassBuildSize;
use crate::scene::level::grass_dither::create_grass_dither;

/// What one storage binding holds at least on every adapter the renderer starts on: 128 MiB.
const BINDING_LIMIT: u64 = 128 << 20;

// `bwdithermap(2, dither)`: `magic4x4` spread over sixteen by sixteen, the coarse cell's value scaled by 254 / 16 and the
// fine one's by a sixteenth of that.
#[test]
fn lays_out_the_engines_dither_column_by_row() {
  let dither: [u32; 256] = create_grass_dither();

  assert_eq!(dither[0], 0);
  // `magic4x4[0][1]`, 14, over a coarse cell of 15.875.
  assert_eq!(dither[1], 222);
  // `magic4x4[1][0]`, 11, coarse steps and nothing finer.
  assert_eq!(dither[16], 175);
  // Nothing coarse, and a sixteenth of `magic4x4[1][0]`'s eleven steps.
  assert_eq!(dither[4 * 16], 11);
  // Both cells at `magic4x4[2][2]`, 15.
  assert_eq!(dither[10 * 16 + 10], 253);
  assert_eq!(dither.iter().max(), Some(&253));
}

#[test]
fn holds_a_cell_a_slot_the_planting_reaches_and_rebuilds_for_a_want_past_it_or_under_half() {
  // The engine's radius of 49 reaches 24 slots each way; its density of 0.6 lays out five by five candidates.
  let held: GrassBuildSize = GrassBuildSize::wanted(24, 25, BINDING_LIMIT);

  assert_eq!(held.cells, 49 * 49);
  assert_eq!(held.per_cell, 25);
  assert_eq!(held.bands, 25);
  assert_eq!(held.capacity, (49 * 49 * 25_u32).next_power_of_two());
  assert!(held.is_fitting(&held));
  assert!(!held.is_fitting(&GrassBuildSize::wanted(26, 25, BINDING_LIMIT)));
  assert!(!held.is_fitting(&GrassBuildSize::wanted(24, 36, BINDING_LIMIT)));
  assert!(held.is_fitting(&GrassBuildSize::wanted(22, 25, BINDING_LIMIT)));
  assert!(!held.is_fitting(&GrassBuildSize::wanted(8, 25, BINDING_LIMIT)));
}

#[test]
fn sways_the_grass_on_its_two_winds_and_stills_it_without_wind() {
  let mut wind: GrassWind = GrassWind::default();

  wind.advance(0.0, true);

  let blowing = wind.advance(0.5, true);

  // Half the normal swing and half the fast one: 0.225 and 0.125 across the ground.
  assert!((blowing.wind_1.length() - 0.225).abs() < 1e-5);
  assert!((blowing.wind_2.length() - 0.125).abs() < 1e-5);
  assert!(blowing.wave_1.w > 0.0);
  assert_eq!(wind.advance(1.0, false).wind_1.length(), 0.0);
}

#[test]
fn carries_the_last_frames_sway_for_the_tufts_motion() {
  let mut wind: GrassWind = GrassWind::default();
  let first = wind.advance(0.0, true);

  // A first frame moved from nowhere.
  assert_eq!(first.previous_wind_1, first.wind_1);
  assert_eq!(first.previous_wave_2, first.wave_2);

  let second = wind.advance(0.5, true);

  assert_eq!(second.previous_wind_1, first.wind_1);
  assert_eq!(second.previous_wind_2, first.wind_2);
  assert_eq!(second.previous_wave_1, first.wave_1);
  assert_ne!(second.wind_1, first.wind_1);
}
