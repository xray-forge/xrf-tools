use std::time::{Duration, Instant};

use xrf_game_platform::{PlatformWindow, PlatformWindowCommand};
use xrf_logging::ProcessStart;

use crate::render::game_view::GameView;
use crate::run::game_cursor::GameCursor;

/// How the game's window first shows, so nothing half built is ever seen: opened hidden, it shows once a frame drawn
/// after a layer was first sent to the view, the loading view over it, is presented, or once its wait runs out, so a
/// renderer that never draws still leaves a window to close.
pub(crate) struct GameStartup {
  /// When play began, which the wait counts from; the log counts from launch.
  started: Instant,
  /// The frames drawn when the layer was first sent, which the window waits past; none before it was sent.
  covered_from: Option<u64>,
  is_shown: bool,
}

impl GameStartup {
  /// How long the window stays hidden waiting for a covered frame.
  const SHOW_WAIT: Duration = Duration::from_secs(30);

  /// The window's start, from now.
  pub(crate) fn new() -> Self {
    Self {
      started: Instant::now(),
      covered_from: None,
      is_shown: false,
    }
  }

  /// Shows `window` once a frame of `view` drawn after a layer was first sent to it is presented, or once the wait ran
  /// out, and takes the cursor for the view unless a layer holds it (`cursor`), which a hidden window cannot.
  pub(crate) fn show_when_covered(&mut self, window: &mut PlatformWindow, view: &GameView, cursor: GameCursor) {
    if self.is_shown {
      return;
    }

    let drawn: u64 = view.count_drawn();

    if view.is_layered() && self.covered_from.is_none() {
      self.covered_from = Some(drawn);
    }

    let is_covered: bool = self.covered_from.is_some_and(|from| drawn > from);
    let is_late: bool = self.started.elapsed() > Self::SHOW_WAIT;

    if !is_covered && !is_late {
      return;
    }

    self.is_shown = true;

    if let Err(error) = window.apply(PlatformWindowCommand::Show) {
      log::warn!("The window did not show: {error}");
    }

    cursor.hand(window);

    log::info!(
      "Window shown {:.1?} after launch{}",
      ProcessStart::get_uptime(),
      if is_covered { "" } else { ", no frame drawn yet" }
    );
  }
}
