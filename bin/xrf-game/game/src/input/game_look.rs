use std::sync::{Mutex, PoisonError};

use xrf_game_movement::MovementHeading;

/// Which way the player looks, owned by presentation (plan decision 26): the main thread turns it from the mouse as
/// each event arrives, the render thread reads it as late as a frame's view can still take it, and the simulation is
/// sent it whole each frame it changed, never the mouse's counts. What the simulation adds to the view later (recoil, a
/// hit) comes back as an offset presentation adds, never in its place.
#[derive(Debug, Default)]
pub struct GameLook {
  state: Mutex<GameLookState>,
}

/// The heading, and whether it changed since the simulation was last sent it.
#[derive(Clone, Copy, Debug, Default)]
struct GameLookState {
  heading: MovementHeading,
  is_changed: bool,
}

impl GameLook {
  /// Which way the player looks now.
  pub fn get(&self) -> MovementHeading {
    self.lock(|state| state.heading)
  }

  /// Turns by the mouse's counts at `sensitivity` radians a count (the input settings'): moved right turns right,
  /// moved down looks down, held short of straight up or down.
  pub fn turn(&self, dx: f32, dy: f32, sensitivity: f32) {
    self.lock(|state| {
      state.heading = state.heading.turned(dx * sensitivity, dy * sensitivity);
      state.is_changed = true;
    });
  }

  /// Faces `heading`, as a pose or a level's start stands the player, which the simulation already holds.
  pub fn face(&self, heading: MovementHeading) {
    self.lock(|state| {
      state.heading = heading;
      state.is_changed = false;
    });
  }

  /// The heading to send the simulation, once after each change; none while unchanged.
  pub fn take_changed(&self) -> Option<MovementHeading> {
    self.lock(|state| std::mem::take(&mut state.is_changed).then_some(state.heading))
  }

  fn lock<T>(&self, read: impl FnOnce(&mut GameLookState) -> T) -> T {
    read(&mut self.state.lock().unwrap_or_else(PoisonError::into_inner))
  }
}
