use std::collections::HashMap;
use std::time::Duration;

use crate::level::effects::campfire::Campfire;
use crate::level::effects::campfire_state::CampfireState;

/// A level's campfires, by their spawned objects' ids, switched together as the view lights them or puts them out:
/// each stands in the switch's state the first time anything asks for it, and every one turns whenever the switch
/// flips, whether or not its particles or its light are drawn.
pub(crate) struct LevelCampfires {
  campfires: HashMap<u16, Campfire>,
  state: CampfireState,
  /// The engine's clock at this frame, which every campfire's turn is timed by.
  now: Duration,
}

impl Default for LevelCampfires {
  fn default() -> Self {
    Self::new()
  }
}

impl LevelCampfires {
  pub(crate) fn new() -> Self {
    Self {
      campfires: HashMap::new(),
      // `m_turned_on`: a campfire spawns lit.
      state: CampfireState::Lit,
      now: Duration::ZERO,
    }
  }

  /// Moves on to `now`, the engine's clock: turns every campfire to the view's switch, `state`, when it flips, and
  /// settles each turn that has taken its time.
  pub(crate) fn prepare(&mut self, state: CampfireState, now: Duration) {
    self.now = now;

    if state != self.state {
      self.state = state;

      for campfire in self.campfires.values_mut() {
        campfire.turn(state, now);
      }
    }

    for campfire in self.campfires.values_mut() {
      campfire.settle(now);
    }
  }

  /// The campfire spawned as an object, standing in the switch's state if nothing asked for it before.
  pub(crate) fn get(&mut self, id: u16) -> &Campfire {
    let state: CampfireState = self.state;

    self.campfires.entry(id).or_insert_with(|| Campfire::new(state))
  }

  /// How much of a campfire's idle light shows this frame.
  pub(crate) fn get_light_share(&mut self, id: u16) -> f32 {
    let now: Duration = self.now;

    self.get(id).get_light_share(now)
  }

  pub(crate) fn get_now(&self) -> Duration {
    self.now
  }
}
