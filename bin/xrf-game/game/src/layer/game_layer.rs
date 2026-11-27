use std::time::{Duration, Instant};

use xrf_error::XrfResult;
use xrf_game_platform::PlatformEvent;

use crate::layer::game_input_fate::GameInputFate;

/// One layer of presentation over a context `C` every layer shares (the game's is `GameLayerContext`): what an agent
/// drives, the developer's console and readouts, a HUD, a menu, the player's own keys. The chain (`GameLayers`) holds
/// them topmost first and runs each part of a frame through them in order; a layer takes part in what it implements.
pub(crate) trait GameLayer<C> {
  /// Appends the input the layer plays itself to `events`, stamped `at` on the platform's clock, for every layer to be
  /// offered as the window's own.
  fn feed(&mut self, _at: Duration, _events: &mut Vec<PlatformEvent>) {}

  /// Takes a platform event the layers over it passed, answering what it did with it.
  fn take_event(&mut self, _event: &PlatformEvent, _context: &mut C) -> GameInputFate {
    GameInputFate::Passed
  }

  /// Lets go of the input it holds, at `at` on the platform's clock: a layer over it seized the keyboard and the mouse.
  fn release_input(&mut self, _at: Duration, _context: &mut C) {}

  /// Moves the layer on a frame, after the frame's events.
  ///
  /// # Errors
  ///
  /// Returns why play cannot go on.
  fn update(&mut self, _context: &mut C) -> XrfResult<()> {
    Ok(())
  }

  /// Draws the layer, after every layer moved on; the layers under it drew first.
  fn draw(&mut self, _context: &mut C) {}

  /// When the layer next has work with no event to wake it, which the main thread's wait ends by; none for a layer
  /// that only answers events.
  fn get_due(&self) -> Option<Instant> {
    None
  }
}
