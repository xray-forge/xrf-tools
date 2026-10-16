use std::time::Duration;

use crate::level::effects::campfire::Campfire;

fn at(milliseconds: u64) -> Duration {
  Duration::from_millis(milliseconds)
}

#[test]
fn plays_and_stops_its_idle_effect_freely_while_settled() {
  let campfire: Campfire = Campfire::new(true);

  assert!(campfire.is_lit());
  assert!(campfire.can_play_idle(at(0)));
  assert!(campfire.can_stop_idle(at(0)));
}

#[test]
fn plays_its_idle_effect_two_seconds_into_a_turn_on() {
  let mut campfire: Campfire = Campfire::new(false);

  campfire.turn(true, at(1000));

  assert!(campfire.is_lit());
  assert!(!campfire.can_play_idle(at(1000)));
  assert!(!campfire.can_play_idle(at(3000)));
  assert!(campfire.can_play_idle(at(3001)));
}

#[test]
fn stops_its_idle_effect_half_a_second_into_a_turn_off() {
  let mut campfire: Campfire = Campfire::new(true);

  campfire.turn(false, at(1000));

  assert!(!campfire.is_lit());
  assert!(!campfire.can_stop_idle(at(1000)));
  assert!(!campfire.can_stop_idle(at(1500)));
  assert!(campfire.can_stop_idle(at(1501)));
}

#[test]
fn ramps_its_light_over_a_turn() {
  let mut campfire: Campfire = Campfire::new(true);

  assert_eq!(campfire.get_light_share(at(0)), 1.0);

  campfire.turn(false, at(1000));

  assert_eq!(campfire.get_light_share(at(1000)), 1.0);
  assert_eq!(campfire.get_light_share(at(2500)), 0.5);

  campfire.settle(at(4000));

  assert_eq!(campfire.get_light_share(at(4000)), 0.0);

  campfire.turn(true, at(5000));

  assert_eq!(campfire.get_light_share(at(5000)), 0.0);
  assert_eq!(campfire.get_light_share(at(6500)), 0.5);
}

#[test]
fn settles_once_the_turn_has_taken_its_time() {
  let mut campfire: Campfire = Campfire::new(true);

  campfire.turn(false, at(1000));
  campfire.settle(at(3999));

  assert_ne!(campfire, Campfire::new(false));

  campfire.settle(at(4000));

  assert_eq!(campfire, Campfire::new(false));
}
