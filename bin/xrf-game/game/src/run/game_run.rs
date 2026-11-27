use std::io::ErrorKind;
use std::sync::Arc;
use std::time::Instant;

use rayon::ThreadPool;
use xrf_error::{XrfError, XrfResult};
use xrf_game_loop::{GameClock, GameLoop, LoopClock, LoopStepper};
use xrf_game_platform::{
  Platform, PlatformSize, PlatformWindow, PlatformWindowCommand, PlatformWindowHandle, PlatformWindowMode,
  PlatformWindowRequest,
};
use xrf_game_settings::{GameSettings, GameWindowMode};
use xrf_job::ExecutionRequest;
use xrf_renderer::RenderWorkers;
use xrf_renderer_host::HostedWindow;
use xrf_world::WorldFedCamera;

use crate::arguments::game_arguments::GameArguments;
#[cfg(feature = "mcp")]
use crate::control::game_control::GameControl;
use crate::directories::game_directories::GameDirectories;
use crate::input::game_bindings::GameBindings;
use crate::render::game_view::GameView;
use crate::run::game_play::GamePlay;
use crate::session::game_session::GameSession;
use crate::state::game_state::GameState;
use crate::state::game_view_feed::GameViewFeed;

/// One run of the game: the session its arguments name opened, then the shell around it (the window, its view, the game
/// loop stepping the session's state, the play loop reading the platform's events) until the player or an agent quits,
/// and everything stopped in the order the window's drop needs.
pub struct GameRun;

impl GameRun {
  /// Plays the level `arguments` name until the player or an agent quits.
  ///
  /// # Errors
  ///
  /// Returns why the game cannot start: an endpoint not built, a session that cannot open (`GameSession::open`), or a
  /// window, renderer, loop or endpoint that cannot start.
  pub fn run(arguments: &GameArguments, settings: &GameSettings, directories: &GameDirectories) -> XrfResult<()> {
    if cfg!(not(feature = "mcp")) && arguments.mcp {
      return Err(XrfError::new_not_implemented_error(
        "this build has no control endpoint; build with the mcp feature",
      ));
    }

    let pool: Arc<ThreadPool> = Arc::new(ExecutionRequest::Auto.resolve().build_pool()?);
    let GameSession {
      level_name,
      source,
      state,
    } = GameSession::open(arguments, &pool)?;
    let mut platform: Platform = Platform::start().map_err(Self::create_platform_error)?;
    let mut window: PlatformWindow = platform
      .open_window(&PlatformWindowRequest {
        title: format!("XRF: {level_name}"),
        size: PlatformSize {
          width: settings.window.width,
          height: settings.window.height,
        },
        mode: match settings.window.mode {
          GameWindowMode::Windowed => PlatformWindowMode::Windowed,
          GameWindowMode::Fullscreen => PlatformWindowMode::Fullscreen,
        },
      })
      .map_err(Self::create_platform_error)?;
    let hosted: HostedWindow<PlatformWindowHandle> = window.create_hosted().map_err(Self::create_platform_error)?;
    let view: GameView = GameView::start(
      RenderWorkers::new(Arc::clone(&pool)),
      settings,
      directories.get_cache().as_ref(),
    );

    view.attach(hosted.get_host(), &window, settings);

    let clock: LoopClock = LoopClock::new(Instant::now(), LoopClock::DEFAULT_TICK_RATE);
    let game_loop: GameLoop<GameState> =
      GameLoop::start(LoopStepper::new(clock.get_step(), GameClock::default(), state), clock)?;

    view.show(
      source,
      Arc::new(GameViewFeed::new(
        Arc::clone(game_loop.get_feed()),
        WorldFedCamera::default(),
      )),
    );
    window
      .apply(PlatformWindowCommand::GrabCursor { is_grabbed: true })
      .map_err(Self::create_platform_error)?;

    GamePlay {
      window: &window,
      view: &view,
      commands: game_loop.get_sender(),
      bindings: GameBindings::default(),
      #[cfg(feature = "mcp")]
      feed: Arc::clone(game_loop.get_feed()),
      #[cfg(feature = "mcp")]
      control: arguments
        .mcp
        .then(|| GameControl::start(&level_name, directories.get_captures_directory()))
        .transpose()?,
    }
    .run(&mut platform);

    // Stopped in the order the window's drop needs: the loop, the view, then the window.
    drop(game_loop);
    view.stop();
    drop(hosted);
    drop(window);

    Ok(())
  }

  fn create_platform_error(message: String) -> XrfError {
    XrfError::new_io_error(message, ErrorKind::Other)
  }
}
