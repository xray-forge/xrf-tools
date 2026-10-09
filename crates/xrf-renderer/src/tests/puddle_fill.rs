use crate::scene::level::puddle_fill::PuddleFill;

// A view first seeing rain starts with full puddles, as stormy as it falls, and with none where it is dry; puddles
// fill while it rains, faster the harder, and drain slower than they filled.
#[test]
fn fills_while_it_rains_and_drains_after() {
  let mut light: PuddleFill = PuddleFill::default();
  let mut heavy: PuddleFill = PuddleFill::default();
  let mut dry: PuddleFill = PuddleFill::default();

  assert_eq!(dry.advance(0.0, 0.0, 1.0), (0.0, 0.0, 0.0));
  assert_eq!(light.advance(0.0, 0.0, 1.0), (0.0, 0.0, 0.0));
  assert_eq!(heavy.advance(0.0, 0.0, 1.0), (0.0, 0.0, 0.0));

  // The first rain fills them whole, even on a paused clock; then they drain and fill again, faster the harder.
  assert_eq!(light.advance(10.0, 0.2, 0.0).0, 1.0);
  assert_eq!(heavy.advance(10.0, 1.0, 1.0).0, 1.0);

  let (drained, _, _) = heavy.advance(1_000.0, 0.0, 1.0);

  assert!(drained < 1.0 && drained > 0.0);
  assert_eq!(light.advance(1_000.0, 0.0, 1.0).0, drained);

  let (lightly, _, _) = light.advance(1_010.0, 0.2, 1.0);
  let (heavily, stormy, _) = heavy.advance(1_010.0, 1.0, 1.0);

  assert!(lightly > drained && heavily > lightly);
  assert!(stormy > 0.0 && stormy < 1.0);
  assert_eq!(heavy.advance(100_000.0, 0.0, 1.0).0, 0.0);
  assert_eq!(PuddleFill::default().advance(0.0, 0.3, 1.0), (1.0, 0.0, 0.0));
}

// The storm follows a heavy rain slowly, and a light rain never makes one.
#[test]
fn follows_a_storm_slowly() {
  let mut fill: PuddleFill = PuddleFill::default();

  fill.advance(0.0, 0.4, 1.0);

  assert_eq!(fill.advance(1_000.0, 0.4, 1.0).1, 0.0);
  assert!(fill.advance(1_001.0, 1.0, 1.0).1 < 0.1);
  assert!(fill.advance(2_000.0, 1.0, 1.0).1 > 0.99);
}

// A long rain soaks the level slowly, the harder the faster, and it dries slower still; the first rain fills the
// puddles whole without soaking it.
#[test]
fn soaks_over_a_long_rain() {
  let mut light: PuddleFill = PuddleFill::default();
  let mut heavy: PuddleFill = PuddleFill::default();

  assert_eq!(light.advance(0.0, 0.3, 1.0).2, 0.0);
  heavy.advance(0.0, 0.3, 1.0);

  let lightly: f32 = light.advance(1_000.0, 0.3, 1.0).2;
  let heavily: f32 = heavy.advance(1_000.0, 1.0, 1.0).2;

  assert!(lightly > 0.0 && heavily > lightly && heavily < 1.0);
  assert_eq!(heavy.advance(10_000.0, 1.0, 1.0).2, 1.0);

  let dried: f32 = heavy.advance(11_000.0, 0.0, 1.0).2;

  assert!(dried < 1.0 && dried > 0.8);
}
