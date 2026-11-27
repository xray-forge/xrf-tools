use std::time::{Duration, Instant};

use xrf_game_developer::DeveloperLoading;
use xrf_game_platform::{PlatformWindow, PlatformWindowCommand};
use xrf_logging::ProcessStart;
use xrf_renderer::RenderLoadReport;

use crate::render::game_view::GameView;

/// How the game comes up, so its level is never seen half built: the window, opened hidden, shows once a frame with the
/// loading view over it was drawn, and the loading view covers the level until the renderer reports it ready, every
/// sector, particle and texture settled, the player's input held meanwhile. Either gives way at its deadline, so a
/// renderer that never draws or never settles still leaves a window to play or close.
pub(crate) struct GameStartup {
  level: String,
  /// When play began, which the waits count from; the log counts from launch.
  started: Instant,
  /// The frames drawn when the loading view was first sent, which the window waits past; none before it was sent.
  covered_from: Option<u64>,
  is_shown: bool,
  is_loading: bool,
}

impl GameStartup {
  /// How long the window stays hidden waiting for a covered frame.
  const SHOW_WAIT: Duration = Duration::from_secs(30);

  /// How long the loading view covers the level waiting for it to be ready.
  const LOAD_WAIT: Duration = Duration::from_secs(60);

  /// The start of the game showing `level`, from now.
  pub(crate) fn new(level: &str) -> Self {
    Self {
      level: level.to_owned(),
      started: Instant::now(),
      covered_from: None,
      is_shown: false,
      is_loading: true,
    }
  }

  /// Whether the loading view still covers the level, the player's input held.
  pub(crate) fn is_loading(&self) -> bool {
    self.is_loading
  }

  /// The loading view to draw over `view`'s level, none once it is ready or its wait ran out.
  pub(crate) fn describe_loading(&mut self, view: &GameView) -> Option<DeveloperLoading> {
    if !self.is_loading {
      return None;
    }

    if view.is_ready() {
      self.is_loading = false;
      log::info!(
        "Level '{}' ready {:.1?} after launch",
        self.level,
        ProcessStart::get_uptime()
      );

      return None;
    }

    if self.started.elapsed() > Self::LOAD_WAIT {
      self.is_loading = false;
      log::warn!(
        "Level '{}' not ready after {:?}; the loading view gives way",
        self.level,
        Self::LOAD_WAIT
      );

      return None;
    }

    Some(Self::create_loading(&self.level, view.get_load()))
  }

  /// Shows `window` once a frame drawn after the loading view was first `sent` is presented, or once the wait ran out,
  /// and takes the cursor for the view unless the console has it, which a hidden window cannot.
  pub(crate) fn show_when_covered(
    &mut self,
    window: &mut PlatformWindow,
    view: &GameView,
    sent: bool,
    is_console_open: bool,
  ) {
    if self.is_shown {
      return;
    }

    let drawn: u64 = view.count_drawn();

    if sent && self.covered_from.is_none() {
      self.covered_from = Some(drawn);
    }

    let is_covered: bool = self.covered_from.is_some_and(|from| drawn > from);
    let is_late: bool = self.started.elapsed() > Self::SHOW_WAIT;

    if !is_covered && !is_late {
      return;
    }

    self.is_shown = true;

    for command in [
      PlatformWindowCommand::Show,
      PlatformWindowCommand::GrabCursor {
        is_grabbed: !is_console_open,
      },
    ] {
      if let Err(error) = window.apply(command) {
        log::warn!("The window did not take {command:?}: {error}");
      }
    }

    log::info!(
      "Window shown {:.1?} after launch{}",
      ProcessStart::get_uptime(),
      if is_covered { "" } else { ", no frame drawn yet" }
    );
  }

  /// The loading view of `level` as far as `load` reached: sectors and textures together, none counted before the
  /// renderer reports.
  pub(crate) fn create_loading(level: &str, load: Option<RenderLoadReport>) -> DeveloperLoading {
    let Some(load) = load else {
      return DeveloperLoading {
        title: format!("Loading {level}"),
        progress: 0.0,
        detail: String::from("Starting the renderer"),
      };
    };
    let done: u32 = load.sectors + load.textures;
    let total: u32 = load.sectors_total + load.textures_total;

    DeveloperLoading {
      title: format!("Loading {level}"),
      progress: if total == 0 { 0.0 } else { done as f32 / total as f32 },
      detail: format!(
        "Sectors {} of {}, textures {} of {}",
        load.sectors, load.sectors_total, load.textures, load.textures_total
      ),
    }
  }
}
