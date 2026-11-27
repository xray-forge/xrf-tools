use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_game_developer::DeveloperLoading;
use xrf_logging::ProcessStart;
use xrf_renderer::RenderLoadReport;

use crate::load::game_load_progress::GameLoadProgress;
use crate::load::game_load_stage::GameLoadStage;
use crate::render::game_view::GameView;

/// One load of a level as play shows it: its progress, shared with whoever does its work, and the loading view over the
/// level until every stage ended, the player's input held meanwhile. Play holds one while a load runs: the game's
/// launch, and later a new game, a saved game loaded, or the player travelling to another level. The stages the view
/// alone sees, the renderer's first frame and the level's streaming, are followed here.
pub(crate) struct GameLoading {
  level: String,
  progress: Arc<GameLoadProgress>,
  started: Instant,
}

impl GameLoading {
  /// How long the loading view covers the level before it gives way, so a renderer that never settles still leaves a
  /// level to play.
  const WAIT: Duration = Duration::from_secs(60);

  /// The load of `level`, its stages reported to `progress`.
  pub(crate) fn new(level: &str, progress: Arc<GameLoadProgress>) -> Self {
    Self {
      level: level.to_owned(),
      progress,
      started: Instant::now(),
    }
  }

  /// Follows the renderer's stages from `view` and answers the loading view to draw; none once the load is done or its
  /// wait ran out, logged.
  pub(crate) fn update(&mut self, view: &GameView) -> Option<DeveloperLoading> {
    if view.count_drawn() > 0 {
      self.progress.finish(GameLoadStage::Renderer);
    }

    if let Some(load) = view.get_load() {
      self
        .progress
        .advance(GameLoadStage::Streaming, Self::get_streamed(&load));
    }

    if view.is_ready() {
      self.progress.finish(GameLoadStage::Streaming);
    }

    if self.progress.is_done() {
      log::info!(
        "Level '{}' loaded {:.1?} after launch",
        self.level,
        ProcessStart::get_uptime()
      );

      return None;
    }

    if self.started.elapsed() > Self::WAIT {
      log::warn!(
        "Level '{}' not loaded after {:?}, still {:?}; the loading view gives way",
        self.level,
        Self::WAIT,
        self.progress.list_running()
      );

      return None;
    }

    Some(self.describe(view.get_load()))
  }

  /// The loading view as the load stands: the whole's progress, and the stages running, the streaming's counts with it.
  pub(crate) fn describe(&self, load: Option<RenderLoadReport>) -> DeveloperLoading {
    let running: Vec<String> = self
      .progress
      .list_running()
      .into_iter()
      .map(|stage| match load {
        Some(load) if stage == GameLoadStage::Streaming => format!(
          "{}: sectors {} of {}, textures {} of {}",
          stage.get_title(),
          load.sectors,
          load.sectors_total,
          load.textures,
          load.textures_total
        ),
        _ => String::from(stage.get_title()),
      })
      .collect();

    DeveloperLoading {
      title: format!("Loading {}", self.level),
      progress: self.progress.get_progress(),
      detail: running.join(" · "),
    }
  }

  /// How far the level's streaming came: sectors and textures together.
  fn get_streamed(load: &RenderLoadReport) -> f32 {
    let total: u32 = load.sectors_total + load.textures_total;

    if total == 0 {
      return 0.0;
    }

    (load.sectors + load.textures) as f32 / total as f32
  }
}
