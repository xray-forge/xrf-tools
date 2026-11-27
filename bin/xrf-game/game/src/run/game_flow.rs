use std::sync::Arc;

use rayon::ThreadPool;
use xrf_error::XrfResult;
use xrf_game_developer::DeveloperLoading;

use crate::developer::game_developer::GameDeveloper;
use crate::input::game_look::GameLook;
use crate::load::game_load_progress::GameLoadProgress;
use crate::load::game_loading::GameLoading;
use crate::load::game_opened::GameOpened;
use crate::load::game_opening::GameOpening;
use crate::render::game_view::GameView;
use crate::run::game_flow_stage::GameFlowStage;
use crate::run::game_running::GameRunning;
use crate::session::game_load_request::GameLoadRequest;

/// What the main thread runs between the window's frames: a load while one runs (its view drawn over the window, the
/// player held, the game opening on a thread of its own and then running under the view while its level streams), and
/// the game played once the load is done. The main thread never waits on a load; it draws its view each frame.
// todo: a menu to show with no game, once the game has one: the flow's Menu stage holds nothing yet.
pub(crate) struct GameFlow {
  loading: Option<GameLoading>,
  opening: Option<GameOpening>,
  running: Option<GameRunning>,
  /// Which way the player looks, which presentation turns and the view draws, whichever game runs.
  look: Arc<GameLook>,
}

impl GameFlow {
  /// Starts loading what `request` asks, its reads on `pool` and its stages reported to `progress`, which the loading
  /// view shows.
  ///
  /// # Errors
  ///
  /// Returns why the load's thread cannot be started.
  pub(crate) fn launch(
    request: &GameLoadRequest,
    pool: Arc<ThreadPool>,
    progress: Arc<GameLoadProgress>,
  ) -> XrfResult<Self> {
    Ok(Self {
      loading: Some(GameLoading::new(&request.level, Arc::clone(&progress))),
      opening: Some(GameOpening::start(request.clone(), pool, progress)?),
      running: None,
      look: Arc::new(GameLook::default()),
    })
  }

  pub(crate) fn get_look(&self) -> &Arc<GameLook> {
    &self.look
  }

  pub(crate) fn get_stage(&self) -> GameFlowStage {
    match (&self.loading, &self.running) {
      (Some(_), _) => GameFlowStage::Loading,
      (None, Some(_)) => GameFlowStage::Playing,
      (None, None) => GameFlowStage::Menu,
    }
  }

  /// The game running, under the loading view or played; none while it opens.
  pub(crate) fn get_running(&self) -> Option<&GameRunning> {
    self.running.as_ref()
  }

  pub(crate) fn get_running_mut(&mut self) -> Option<&mut GameRunning> {
    self.running.as_mut()
  }

  /// Moves the load on and answers its view to draw: what the opening told taken (its picture to `developer`, its
  /// game started and shown in `view`), the renderer's stages followed; once every stage ended, the game is played.
  ///
  /// # Errors
  ///
  /// Returns why the game did not open, or its loop did not start.
  pub(crate) fn advance(
    &mut self,
    view: &GameView,
    developer: &mut GameDeveloper,
  ) -> XrfResult<Option<DeveloperLoading>> {
    let opened: Vec<GameOpened> = self.opening.as_ref().map(GameOpening::take).unwrap_or_default();

    for event in opened {
      match event {
        GameOpened::Picture(picture) => developer.set_loading_picture(&picture),
        GameOpened::Session(session) => {
          self.opening = None;
          self.running = Some(GameRunning::start((*session)?, view, &self.look)?);
        }
      }
    }

    let Some(loading) = &mut self.loading else {
      return Ok(None);
    };
    let shown: Option<DeveloperLoading> = loading.update(view);

    // Done, or its wait ran out: the view gives way, to the game, or to none until a slow game opens.
    if shown.is_none() {
      self.loading = None;
    }

    Ok(shown)
  }
}
