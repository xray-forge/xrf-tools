use crate::scene::level::rain_wetness::RainWetness;

// Every rain starts on dry ground: the level builds up towards the rain's density while it rains, settles there, and
// dries towards a lighter rain or none after.
#[test]
fn builds_up_towards_the_rain_and_dries_after() {
  let mut wetness: RainWetness = RainWetness::default();

  assert_eq!(wetness.get(), 0.0);
  assert_eq!(wetness.advance(10.0, 0.5, 1.0, RainWetness::ENGINE_BUILD_UP), 0.0);

  let wetter: f32 = wetness.advance(20.0, 0.5, 1.0, RainWetness::ENGINE_BUILD_UP);

  assert!(wetter > 0.0 && wetter < 0.5);
  assert_eq!(wetness.advance(10_000.0, 0.5, 1.0, RainWetness::ENGINE_BUILD_UP), 0.5);

  let lighter: f32 = wetness.advance(10_010.0, 0.2, 1.0, RainWetness::ENGINE_BUILD_UP);

  assert!(lighter < 0.5 && lighter > 0.2);
  assert_eq!(wetness.advance(100_000.0, 0.2, 1.0, RainWetness::ENGINE_BUILD_UP), 0.2);
  assert_eq!(wetness.advance(200_000.0, 0.0, 1.0, RainWetness::ENGINE_BUILD_UP), 0.0);
}

// It moves on the weather's clock: still while it stands still, ten times as far where it runs ten times as fast.
#[test]
fn dries_at_the_weather_clocks_pace() {
  let soaked = || {
    let mut wetness: RainWetness = RainWetness::default();

    wetness.advance(0.0, 1.0, 1.0, RainWetness::ENGINE_BUILD_UP);
    wetness.advance(1_000.0, 1.0, 1.0, RainWetness::ENGINE_BUILD_UP);

    wetness
  };

  assert_eq!(soaked().advance(1_100.0, 0.0, 0.0, RainWetness::ENGINE_BUILD_UP), 1.0);

  let dried_slowly: f32 = 1.0 - soaked().advance(1_010.0, 0.0, 1.0, RainWetness::ENGINE_BUILD_UP);
  let dried_fast: f32 = 1.0 - soaked().advance(1_010.0, 0.0, 10.0, RainWetness::ENGINE_BUILD_UP);

  assert!((dried_fast - dried_slowly * 10.0).abs() < 1e-5);
}

// The enhanced rain wets twice as fast as the engine's, and dries as it does.
#[test]
fn wets_twice_as_fast_enhanced() {
  let wetted = |build_up: f32| {
    let mut wetness: RainWetness = RainWetness::default();

    wetness.advance(0.0, 1.0, 1.0, build_up);
    wetness.advance(10.0, 1.0, 1.0, build_up)
  };

  assert!((wetted(RainWetness::ENHANCED_BUILD_UP) - wetted(RainWetness::ENGINE_BUILD_UP) * 2.0).abs() < 1e-6);
}

// A rain wets every surface it reaches alike, to the wet spell's peak; once it stops the peak holds and the share left
// falls as the level dries, to none and no peak when it is dry; a new rain closes the peak on the wetness again.
#[test]
fn keeps_the_wet_spells_peak_to_dry_from() {
  let mut wetness: RainWetness = RainWetness::default();
  let build_up: f32 = RainWetness::ENGINE_BUILD_UP;

  wetness.advance(0.0, 0.8, 1.0, build_up);
  wetness.advance(1_000.0, 0.8, 1.0, build_up);

  assert_eq!((wetness.get_peak(), wetness.get_left()), (0.8, 1.0));

  wetness.advance(1_100.0, 0.0, 1.0, build_up);

  assert_eq!(wetness.get_peak(), 0.8);
  assert!(wetness.get_left() < 1.0 && wetness.get_left() > 0.0);

  wetness.advance(1_120.0, 0.8, 1.0, build_up);

  assert!(wetness.get_peak() < 0.8);
  assert!(wetness.get_left() > (wetness.get() / 0.8));

  wetness.advance(2_000.0, 0.0, 1.0, build_up);

  assert_eq!((wetness.get(), wetness.get_peak(), wetness.get_left()), (0.0, 0.0, 1.0));
}
