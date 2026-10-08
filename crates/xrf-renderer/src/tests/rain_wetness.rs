use crate::scene::level::rain_wetness::RainWetness;

// A view first seeing rain starts as wet as it falls, builds up while it rains and dries after, and stays within
// nothing and one.
#[test]
fn builds_up_while_it_rains_and_dries_after() {
  let mut wetness: RainWetness = RainWetness::default();

  assert_eq!(wetness.get(), 0.0);
  assert_eq!(wetness.advance(10.0, 0.5, 1.0), 0.5);

  let wetter: f32 = wetness.advance(20.0, 0.5, 1.0);

  assert!(wetter > 0.5);
  assert!(wetness.advance(30.0, 0.0, 1.0) < wetter);
  assert_eq!(wetness.advance(100_000.0, 0.0, 1.0), 0.0);
  assert_eq!(wetness.advance(200_000.0, 1.0, 1.0), 1.0);
}

// It moves on the weather's clock: still while the clock is paused, ten times as far where it runs ten times as fast.
#[test]
fn dries_at_the_weather_clocks_pace() {
  let soaked = || {
    let mut wetness: RainWetness = RainWetness::default();

    wetness.advance(0.0, 1.0, 1.0);

    wetness
  };

  assert_eq!(soaked().advance(100.0, 0.0, 0.0), 1.0);

  let dried_slowly: f32 = 1.0 - soaked().advance(10.0, 0.0, 1.0);
  let dried_fast: f32 = 1.0 - soaked().advance(10.0, 0.0, 10.0);

  assert!((dried_fast - dried_slowly * 10.0).abs() < 1e-5);
}
