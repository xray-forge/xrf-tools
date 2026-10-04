use std::collections::HashMap;
use std::time::Instant;

use crate::scene::level::campfire::Campfire;

/// A level's campfires, by their spawned objects' ids, switched together as the view lights them or puts them out:
/// each stands in the switch's state the first time anything asks for it, and every one turns whenever the switch
/// flips, whether or not its particles or its light are drawn.
pub struct LevelCampfires {
  campfires: HashMap<u16, Campfire>,
  is_lit: bool,
  started: Instant,
  /// The viewer's milliseconds at this frame, which every campfire's turn is timed by.
  now: u64,
}

impl Default for LevelCampfires {
  fn default() -> Self {
    Self::new()
  }
}

impl LevelCampfires {
  pub fn new() -> Self {
    Self {
      campfires: HashMap::new(),
      // `m_turned_on`: a campfire spawns lit.
      is_lit: true,
      started: Instant::now(),
      now: 0,
    }
  }

  /// Turns every campfire to the view's switch when it flips, and settles each turn that has taken its time.
  pub fn prepare(&mut self, is_lit: bool) {
    self.prepare_at(is_lit, self.started.elapsed().as_millis() as u64);
  }

  /// The campfire spawned as an object, standing in the switch's state if nothing asked for it before.
  pub fn get(&mut self, id: u16) -> &Campfire {
    let is_lit: bool = self.is_lit;

    self.campfires.entry(id).or_insert_with(|| Campfire::new(is_lit))
  }

  /// How much of a campfire's idle light shows this frame.
  pub fn get_light_share(&mut self, id: u16) -> f32 {
    let now: u64 = self.now;

    self.get(id).get_light_share(now)
  }

  pub fn get_now(&self) -> u64 {
    self.now
  }

  fn prepare_at(&mut self, is_lit: bool, now: u64) {
    self.now = now;

    if is_lit != self.is_lit {
      self.is_lit = is_lit;

      for campfire in self.campfires.values_mut() {
        campfire.turn(is_lit, now);
      }
    }

    for campfire in self.campfires.values_mut() {
      campfire.settle(now);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::LevelCampfires;
  use crate::scene::level::campfire::Campfire;

  #[test]
  fn stands_a_campfire_first_asked_for_in_the_switchs_state() {
    let mut campfires: LevelCampfires = LevelCampfires::new();

    campfires.prepare_at(false, 1000);

    assert_eq!(*campfires.get(3), Campfire::new(false));
  }

  #[test]
  fn turns_every_campfire_as_the_switch_flips_and_settles_them() {
    let mut campfires: LevelCampfires = LevelCampfires::new();

    campfires.prepare_at(true, 0);
    campfires.get(1);
    campfires.get(2);
    campfires.prepare_at(false, 1000);

    assert!(!campfires.get(1).is_lit());
    assert!(campfires.get(2).is_turning());

    campfires.prepare_at(false, 4000);

    assert_eq!(*campfires.get(1), Campfire::new(false));
    assert_eq!(*campfires.get(2), Campfire::new(false));
  }
}
