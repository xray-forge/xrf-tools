use crate::scene::level::rain_wetness::RainWetness;

// A view first seeing rain starts as wet as it falls, builds up while it rains, dries after, and stays within nothing
// and one.
#[test]
fn builds_up_while_it_rains_and_dries_after() {
  let mut wetness: RainWetness = RainWetness::default();

  assert_eq!(wetness.get(), 0.0);
  assert_eq!(wetness.advance(10.0, 0.5), 0.5);

  let wetter: f32 = wetness.advance(20.0, 0.5);

  assert!(wetter > 0.5);
  assert!(wetness.advance(30.0, 0.0) < wetter);
  assert_eq!(wetness.advance(100_000.0, 0.0), 0.0);
  assert_eq!(wetness.advance(200_000.0, 1.0), 1.0);
  assert_eq!(wetness.advance(100.0, 1.0), 1.0);
}
