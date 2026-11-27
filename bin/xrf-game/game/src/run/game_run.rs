#[cfg(feature = "mcp")]
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

use rayon::ThreadPool;
use xrf_error::{XrfError, XrfResult};
use xrf_game_loop::{GameClock, GameLoop, LoopClock, LoopStepper};
#[cfg(feature = "mcp")]
use xrf_game_mcp::GameEndpoint;
use xrf_game_settings::GameSettings;
use xrf_job::ExecutionRequest;
use xrf_logging::LogTail;
use xrf_world::WorldFedCamera;

use crate::arguments::game_arguments::GameArguments;
use crate::console::game_console::GameConsole;
#[cfg(feature = "mcp")]
use crate::control::game_control::GameControl;
use crate::developer::game_developer::GameDeveloper;
use crate::directories::game_directories::GameDirectories;
use crate::input::game_bindings::GameBindings;
use crate::run::game_play::GamePlay;
use crate::run::game_shell::GameShell;
use crate::run::game_startup::GameStartup;
use crate::session::game_session::GameSession;
use crate::state::game_state::GameState;
use crate::state::game_view_feed::GameViewFeed;

/// One run of the game: the session its arguments name opened while the shell starts (the window and the renderer in
/// it), then the game loop stepping the session's state and the play loop reading the platform's events until the
/// player or an agent quits, and everything stopped in the order the window's drop needs, however the run ends.
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
    // The session reads the level on the pool while the main thread starts the shell, which needs no level: the
    // renderer's own start no longer waits for the level's.
    let (session, shell): (XrfResult<GameSession>, XrfResult<GameShell>) = thread::scope(|scope| {
      let opening = scope.spawn(|| GameSession::open(arguments, &pool));
      let shell: XrfResult<GameShell> = GameShell::start(
        arguments.level.as_deref().unwrap_or_default(),
        settings,
        directories,
        &pool,
      );
      let session: XrfResult<GameSession> = opening
        .join()
        .unwrap_or_else(|_| Err(XrfError::new_unexpected_error("opening the game panicked")));

      (session, shell)
    });
    let mut shell: GameShell = shell?;
    let played: XrfResult<()> =
      session.and_then(|session| Self::play(arguments, settings, directories, log, session, &mut shell));

    shell.stop();

    played
  }

  /// Steps `session`'s state on the game loop and plays it in `shell` until the player or an agent quits; the loop is
  /// stopped before the shell is.
  fn play(
    arguments: &GameArguments,
    settings: &GameSettings,
    directories: &GameDirectories,
    log: Arc<LogTail>,
    session: GameSession,
    shell: &mut GameShell,
  ) -> XrfResult<()> {
    let GameSession {
      level_name,
      source,
      state,
      footsteps,
      loading_picture,
    } = session;
    let clock: LoopClock = LoopClock::new(Instant::now(), LoopClock::DEFAULT_TICK_RATE);
    let game_loop: GameLoop<GameState> =
      GameLoop::start(LoopStepper::new(clock.get_step(), GameClock::default(), state), clock)?;

    shell.view.show(
      source,
      Arc::new(GameViewFeed::new(
        Arc::clone(game_loop.get_feed()),
        WorldFedCamera::default(),
      )),
    );

    let mut play: GamePlay<'_> = GamePlay {
      window: &mut shell.window,
      view: &shell.view,
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
      startup: GameStartup::new(&level_name),
    };

    if let Some(picture) = &loading_picture {
      play.developer.set_loading_picture(picture);
    }

    play.run(&mut shell.platform);
    // However play ended, what the console changed is kept for the next run.
    play.console.save_on_quit();

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
}
