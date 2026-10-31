use std::time::Duration;

use crate::level::effects::campfire::Campfire;
use crate::level::effects::campfire_state::CampfireState;
use crate::level::effects::level_campfires::LevelCampfires;

#[test]
fn stands_a_campfire_first_asked_for_in_the_switchs_state() {
  let mut campfires: LevelCampfires = LevelCampfires::new();

  campfires.prepare(CampfireState::Out, Duration::from_millis(1000));

  assert_eq!(*campfires.get(3), Campfire::new(CampfireState::Out));
}

#[test]
fn turns_every_campfire_as_the_switch_flips_and_settles_them() {
  let mut campfires: LevelCampfires = LevelCampfires::new();

  campfires.prepare(CampfireState::Lit, Duration::ZERO);
  campfires.get(1);
  campfires.get(2);
  campfires.prepare(CampfireState::Out, Duration::from_millis(1000));

  assert!(!campfires.get(1).is_lit());
  assert!(campfires.get(2).is_turning());

  campfires.prepare(CampfireState::Out, Duration::from_millis(4000));

  assert_eq!(*campfires.get(1), Campfire::new(CampfireState::Out));
  assert_eq!(*campfires.get(2), Campfire::new(CampfireState::Out));
}
