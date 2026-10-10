use crate::scene::level::rain_wetness::RainWetness;

// Every rain starts on dry ground: the level builds up towards the rain's density while it rains, settles there, and
// dries towards a lighter rain or none after.
#[test]
fn builds_up_towards_the_rain_and_dries_after() {
  let mut wetness: RainWetness = RainWetness::default();

  assert_eq!(wetness.get(), 0.0);
  assert_eq!(wetness.advance(10.0, 0.5, 1.0), 0.0);

  let wetter: f32 = wetness.advance(20.0, 0.5, 1.0);

  assert!(wetter > 0.0 && wetter < 0.5);
  assert_eq!(wetness.advance(10_000.0, 0.5, 1.0), 0.5);

  let lighter: f32 = wetness.advance(10_010.0, 0.2, 1.0);

  assert!(lighter < 0.5 && lighter > 0.2);
  assert_eq!(wetness.advance(100_000.0, 0.2, 1.0), 0.2);
  assert_eq!(wetness.advance(200_000.0, 0.0, 1.0), 0.0);
}

// It moves on the weather's clock: still while it stands still, ten times as far where it runs ten times as fast.
#[test]
fn dries_at_the_weather_clocks_pace() {
  let soaked = || {
    let mut wetness: RainWetness = RainWetness::default();

    wetness.advance(0.0, 1.0, 1.0);
    wetness.advance(1_000.0, 1.0, 1.0);

    wetness
  };

  assert_eq!(soaked().advance(1_100.0, 0.0, 0.0), 1.0);

  let dried_slowly: f32 = 1.0 - soaked().advance(1_010.0, 0.0, 1.0);
  let dried_fast: f32 = 1.0 - soaked().advance(1_010.0, 0.0, 10.0);

  assert!((dried_fast - dried_slowly * 10.0).abs() < 1e-5);
}
