use xrf_game_platform::{PlatformWindow, PlatformWindowCommand};

/// Whom the window's cursor and typed text are for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GameCursor {
  /// The view: the cursor is grabbed and its motion turns the look; no text is typed.
  View,
  /// A layer holding the keyboard and the mouse, as the console does while open: the cursor is free and text is typed.
  Layer,
}

impl GameCursor {
  /// Whom the cursor is for while a layer holds the keyboard and the mouse (`is_seized`) or none does.
  pub(crate) fn of_seized(is_seized: bool) -> Self {
    if is_seized { Self::Layer } else { Self::View }
  }

  /// Hands `window`'s cursor and typed text to whom they are for.
  pub(crate) fn hand(self, window: &mut PlatformWindow) {
    let is_free: bool = self == Self::Layer;

    for command in [
      PlatformWindowCommand::GrabCursor { is_grabbed: !is_free },
      PlatformWindowCommand::AcceptText { is_accepting: is_free },
    ] {
      if let Err(error) = window.apply(command) {
        log::warn!("The window did not take {command:?}: {error}");
      }
    }
  }
}
