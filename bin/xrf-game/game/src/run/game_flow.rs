use std::sync::Arc;

use rayon::ThreadPool;
use xrf_error::XrfResult;
use xrf_game_developer::{DeveloperLoading, DeveloperPicture};
use xrf_game_settings::GameInputSettings;

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
  /// The load's view as it stood when the flow last moved on, none once the load gave way.
  loading_view: Option<DeveloperLoading>,
  /// The load's picture, from its arrival until the layer drawing the loading view takes it.
  loading_picture: Option<DeveloperPicture>,
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
      loading_view: None,
      loading_picture: None,
    })
  }

  /// The load's view to draw this frame, none while no load covers the level.
  pub(crate) fn get_loading_view(&self) -> Option<&DeveloperLoading> {
    self.loading_view.as_ref()
  }

  /// The picture the loading view shows, once after it arrived.
  pub(crate) fn take_loading_picture(&mut self) -> Option<DeveloperPicture> {
    self.loading_picture.take()
  }

  pub(crate) fn get_look(&self) -> &Arc<GameLook> {
    &self.look
  }

  /// While the game replays a recorded session, faces the look where the session faced the player, so the view shows
  /// what was recorded whatever the mouse does, and the player takes over at the replay's end facing as it left them.
  pub(crate) fn follow_replay(&self) {
    if let Some(heading) = self.running.as_ref().and_then(GameRunning::get_replayed_heading) {
      self.look.face(heading);
    }
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

  /// Moves the load on a frame: what the opening told taken (its picture kept for the loading view, its game started
  /// and shown in `view`, its keys bound with `input`'s), the renderer's stages followed and the load's view kept to
  /// draw; once every stage ended, the game is played.
  ///
  /// # Errors
  ///
  /// Returns why the game did not open, or its loop did not start.
  pub(crate) fn advance(&mut self, view: &GameView, input: &GameInputSettings) -> XrfResult<()> {
    let opened: Vec<GameOpened> = self.opening.as_ref().map(GameOpening::take).unwrap_or_default();

    for event in opened {
      match event {
        GameOpened::Picture(picture) => self.loading_picture = Some(picture),
        GameOpened::Session(session) => {
          self.opening = None;
          self.running = Some(GameRunning::start((*session)?, view, &self.look, input)?);
        }
      }
    }

    self.loading_view = self.loading.as_mut().and_then(|loading| loading.update(view));

    // Done, or its wait ran out: the view gives way, to the game, or to none until a slow game opens.
    if self.loading_view.is_none() {
      self.loading = None;
    }

    Ok(())
  }
}
