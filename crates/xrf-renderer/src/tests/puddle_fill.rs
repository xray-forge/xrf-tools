use crate::scene::level::puddle_fill::PuddleFill;

// Every rain starts with empty puddles: they fill while it rains, faster the harder, and drain slower than they filled,
// down to nothing.
#[test]
fn fills_while_it_rains_and_drains_after() {
  let mut light: PuddleFill = PuddleFill::default();
  let mut heavy: PuddleFill = PuddleFill::default();

  assert_eq!(light.advance(0.0, 0.2, 1.0), (0.0, 0.0, 0.0));
  assert_eq!(heavy.advance(0.0, 1.0, 1.0), (0.0, 0.0, 0.0));

  let (lightly, _, _) = light.advance(10.0, 0.2, 1.0);
  let (heavily, stormy, _) = heavy.advance(10.0, 1.0, 1.0);

  assert!(lightly > 0.0 && heavily > lightly && heavily < 1.0);
  assert!(stormy > 0.0 && stormy < 1.0);
  assert_eq!(heavy.advance(1_000.0, 1.0, 1.0).0, 1.0);

  let (drained, _, _) = heavy.advance(1_010.0, 0.0, 1.0);

  assert!(drained < 1.0 && drained > 0.99);
  assert_eq!(heavy.advance(100_000.0, 0.0, 1.0).0, 0.0);
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

// The storm eases as the puddles drain, so the puddles a downpour made outlast it once the rain lightens.
#[test]
fn eases_off_a_storm_as_puddles_drain() {
  let mut fill: PuddleFill = PuddleFill::default();

  fill.advance(0.0, 1.0, 1.0);
  fill.advance(2_000.0, 1.0, 1.0);

  let eased: f32 = fill.advance(2_100.0, 0.4, 1.0).1;

  assert!(eased > 0.9 && eased < 1.0);
  assert_eq!(fill.advance(10_000.0, 0.4, 1.0).1, 0.0);
}

// A long rain soaks the level slowly, the harder the faster, and it dries slower still.
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
