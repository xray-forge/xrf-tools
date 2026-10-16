use std::collections::HashMap;
use std::time::Duration;

use crate::level::campfire::Campfire;

/// A level's campfires, by their spawned objects' ids, switched together as the view lights them or puts them out:
/// each stands in the switch's state the first time anything asks for it, and every one turns whenever the switch
/// flips, whether or not its particles or its light are drawn.
pub struct LevelCampfires {
  campfires: HashMap<u16, Campfire>,
  is_lit: bool,
  /// The level's clock at this frame, which every campfire's turn is timed by.
  now: Duration,
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
      now: Duration::ZERO,
    }
  }

  /// Moves on to `now`, the level's clock: turns every campfire to the view's switch when it flips, and settles each
  /// turn that has taken its time.
  pub fn prepare(&mut self, is_lit: bool, now: Duration) {
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

  /// The campfire spawned as an object, standing in the switch's state if nothing asked for it before.
  pub fn get(&mut self, id: u16) -> &Campfire {
    let is_lit: bool = self.is_lit;

    self.campfires.entry(id).or_insert_with(|| Campfire::new(is_lit))
  }

  /// How much of a campfire's idle light shows this frame.
  pub fn get_light_share(&mut self, id: u16) -> f32 {
    let now: Duration = self.now;

    self.get(id).get_light_share(now)
  }

  pub fn get_now(&self) -> Duration {
    self.now
  }
}
