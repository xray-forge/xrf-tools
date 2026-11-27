use std::io::ErrorKind;
#[cfg(feature = "mcp")]
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use rayon::ThreadPool;
use xrf_error::{XrfError, XrfResult};
use xrf_game_loop::{GameClock, GameLoop, LoopClock, LoopStepper};
#[cfg(feature = "mcp")]
use xrf_game_mcp::GameEndpoint;
use xrf_game_platform::{
  Platform, PlatformSize, PlatformWindow, PlatformWindowHandle, PlatformWindowMode, PlatformWindowRequest,
};
use xrf_game_settings::{GameSettings, GameWindowMode};
use xrf_job::ExecutionRequest;
use xrf_logging::LogTail;
use xrf_renderer::RenderWorkers;
use xrf_renderer_host::HostedWindow;
use xrf_world::WorldFedCamera;

use crate::arguments::game_arguments::GameArguments;
use crate::console::game_console::GameConsole;
#[cfg(feature = "mcp")]
use crate::control::game_control::GameControl;
use crate::developer::game_developer::GameDeveloper;
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
  /// Plays the level `arguments` name until the player or an agent quits, its console showing `log`'s records.
  ///
  /// # Errors
  ///
  /// Returns why the game cannot start: an endpoint not built, a session that cannot open (`GameSession::open`), or a
  /// window, renderer, loop or endpoint that cannot start.
  pub fn run(
    arguments: &GameArguments,
    settings: &GameSettings,
    directories: &GameDirectories,
    log: Arc<LogTail>,
  ) -> XrfResult<()> {
    if cfg!(not(feature = "mcp")) && (arguments.mcp || arguments.listen) {
      return Err(XrfError::new_not_implemented_error(
        "this build has no control endpoint; build with the mcp feature",
      ));
    }

    let pool: Arc<ThreadPool> = Arc::new(ExecutionRequest::Auto.resolve().build_pool()?);
    let GameSession {
      level_name,
      source,
      state,
      footsteps,
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

    let mut play: GamePlay<'_> = GamePlay {
      window: &mut window,
      view: &view,
      commands: game_loop.get_sender(),
      bindings: GameBindings::default(),
      feed: Arc::clone(game_loop.get_feed()),
      developer: GameDeveloper::default(),
      console: GameConsole::new(settings.clone(), directories.get_settings_files()),
      log,
      footsteps,
      #[cfg(feature = "mcp")]
      control: Self::start_control(arguments, &level_name, directories)?,
      is_quitting: false,
      show_by: Some(Instant::now() + GamePlay::SHOW_WAIT),
    };

    play.run(&mut platform);
    // However play ended, what the console changed is kept for the next run.
    play.console.save_on_quit();
    drop(play);

    // Stopped in the order the window's drop needs: the loop, the view, then the window.
    drop(game_loop);
    view.stop();
    drop(hosted);
    drop(window);

    Ok(())
  }

  /// The control endpoint `arguments` ask for: on standard input for the bridge's child (`--mcp`), listening for a
  /// bridge to attach to a game started by hand (`--listen`), or none.
  #[cfg(feature = "mcp")]
  fn start_control(
    arguments: &GameArguments,
    level_name: &str,
    directories: &GameDirectories,
  ) -> XrfResult<Option<GameControl>> {
    let endpoint: GameEndpoint = if arguments.mcp {
      GameEndpoint::start()?
    } else if arguments.listen {
      let directory: PathBuf = directories
        .get_attach_directory()
        .ok_or_else(|| XrfError::new_not_found_error("the game has no folder to write how to attach to it"))?;

      GameEndpoint::listen(&directory, level_name)?
    } else {
      return Ok(None);
    };

    Ok(Some(GameControl::new(
      endpoint,
      level_name,
      directories.get_captures_directory(),
    )))
  }

  fn create_platform_error(message: String) -> XrfError {
    XrfError::new_io_error(message, ErrorKind::Other)
  }
}
