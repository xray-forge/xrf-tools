use std::time::{Duration, Instant};

use xrf_error::XrfResult;
use xrf_game_platform::PlatformEvent;

use crate::layer::game_input_fate::GameInputFate;
use crate::layer::game_layer::GameLayer;

/// The chain of presentation layers over a context `C`, topmost first. Input runs down it: each event is offered to the
/// layers in order until one keeps it. Each then moves on a frame in the same order, and they draw from the bottom up,
/// so a layer draws over those under it. A new layer is its type and a `push` where the chain is built.
///
/// The chain itself keeps which layers hold the keyboard and the mouse, by what each answered, so a layer that seizes
/// while it holds them, or yields while it does not, changes nothing.
pub(crate) struct GameLayers<C> {
  entries: Vec<GameLayerEntry<C>>,
}

/// A layer in the chain, and whether it holds the keyboard and the mouse.
struct GameLayerEntry<C> {
  layer: Box<dyn GameLayer<C>>,
  is_holding: bool,
}

impl<C> Default for GameLayers<C> {
  fn default() -> Self {
    Self { entries: Vec::new() }
  }
}

impl<C> GameLayers<C> {
  /// Adds `layer` under those added before.
  pub(crate) fn push(&mut self, layer: impl GameLayer<C> + 'static) {
    self.entries.push(GameLayerEntry {
      layer: Box::new(layer),
      is_holding: false,
    });
  }

  /// Whether a layer holds the keyboard and the mouse, so the cursor is free for it rather than turning the view.
  pub(crate) fn is_seized(&self) -> bool {
    self.entries.iter().any(|entry| entry.is_holding)
  }

  /// Appends the input the layers play themselves to `events`, stamped `at` on the platform's clock.
  pub(crate) fn feed(&mut self, at: Duration, events: &mut Vec<PlatformEvent>) {
    for entry in &mut self.entries {
      entry.layer.feed(at, events);
    }
  }

  /// Offers `event` to each layer from the top until one keeps it, answering whether one did. The layers under one
  /// that seized the keyboard and the mouse let go of what they hold, once for each time it took them.
  pub(crate) fn take_event(&mut self, event: &PlatformEvent, context: &mut C) -> bool {
    for index in 0..self.entries.len() {
      let entry: &mut GameLayerEntry<C> = &mut self.entries[index];

      match entry.layer.take_event(event, context) {
        GameInputFate::Passed => continue,
        GameInputFate::Kept => {}
        GameInputFate::Seized => {
          if !std::mem::replace(&mut entry.is_holding, true) {
            for under in &mut self.entries[index + 1..] {
              under.layer.release_input(event.at, context);
            }
          }
        }
        GameInputFate::Yielded => entry.is_holding = false,
      }

      return true;
    }

    false
  }

  /// Moves every layer on a frame, from the top.
  ///
  /// # Errors
  ///
  /// Returns the first layer's reason play cannot go on; the layers under it are not moved.
  pub(crate) fn update(&mut self, context: &mut C) -> XrfResult<()> {
    self
      .entries
      .iter_mut()
      .try_for_each(|entry| entry.layer.update(context))
  }

  /// Draws every layer, from the bottom.
  pub(crate) fn draw(&mut self, context: &mut C) {
    for entry in self.entries.iter_mut().rev() {
      entry.layer.draw(context);
    }
  }

  /// When the first of the layers next has work with no event to wake it; none while all only answer events.
  pub(crate) fn get_due(&self) -> Option<Instant> {
    self.entries.iter().filter_map(|entry| entry.layer.get_due()).min()
  }
}
