#[cfg(feature = "mcp")]
use std::path::PathBuf;
use std::sync::Arc;

use rayon::ThreadPool;
use xrf_error::{XrfError, XrfResult};
#[cfg(feature = "mcp")]
use xrf_game_mcp::GameEndpoint;
use xrf_game_settings::{GameSettings, GameSettingsStore};
use xrf_job::ExecutionRequest;
use xrf_logging::LogTail;

use crate::arguments::game_arguments::GameArguments;
#[cfg(feature = "mcp")]
use crate::control::game_control::GameControl;
use crate::developer::game_developer::GameDeveloper;
use crate::directories::game_directories::GameDirectories;
use crate::layer::game_layer_context::GameLayerContext;
use crate::layer::game_layers::GameLayers;
use crate::layer::game_player_layer::GamePlayerLayer;
use crate::load::game_load_progress::GameLoadProgress;
use crate::load::game_load_stage::GameLoadStage;
use crate::run::game_flow::GameFlow;
use crate::run::game_play::GamePlay;
use crate::run::game_shell::GameShell;
use crate::session::game_load_request::GameLoadRequest;

/// One run of the game: the load its arguments ask for started on a thread of its own, then the shell (the window and
/// the renderer in it) on the main thread, which plays through the chain of presentation layers built here (the control
/// endpoint where asked for, the developer layer, the player's own), drawing the load's view while it runs and the game
/// once loaded, until the player or an agent quits; everything stopped in the order the window's drop needs, however
/// the run ends.
pub struct GameRun;

impl GameRun {
  /// Plays the level `arguments` name until the player or an agent quits, its console showing `log`'s records.
  ///
  /// # Errors
  ///
  /// Returns why the game cannot start: an endpoint not built, no tree or level named, a game that does not open
  /// (`GameSession::open`), or a window, renderer, loop or endpoint that cannot start.
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

    let request: GameLoadRequest = arguments.read_request()?;
    let pool: Arc<ThreadPool> = Arc::new(ExecutionRequest::Auto.resolve().build_pool()?);
    let progress: Arc<GameLoadProgress> = Arc::new(GameLoadProgress::new(&GameLoadStage::LAUNCH));
    // The load reads on the pool while the main thread starts the shell, which needs no level: the renderer's own
    // start no longer waits for the level's.
    let flow: GameFlow = GameFlow::launch(&request, Arc::clone(&pool), Arc::clone(&progress))?;
    #[cfg(feature = "mcp")]
    let control: Option<GameControl> = Self::start_control(arguments, &request.level, directories)?;

    progress.begin(GameLoadStage::Renderer);

    let mut shell: GameShell = GameShell::start(&request.level, settings, directories, &pool)?;
    // Play, and with it the flow and the game's loop, ends before the shell is stopped, however it ends.
    let played: XrfResult<()> = {
      // The presentation layers, topmost first: input runs down them, and they draw from the bottom up.
      let mut layers: GameLayers<GameLayerContext<'_>> = GameLayers::default();

      #[cfg(feature = "mcp")]
      if let Some(control) = control {
        layers.push(control);
      }

      layers.push(GameDeveloper::new(log));
      layers.push(GamePlayerLayer);

      let mut play: GamePlay<'_> = GamePlay::new(
        GameLayerContext::new(
          &mut shell.window,
          &shell.view,
          // Play owns the run's settings; the console and, through it, the bridge borrow them a line at a time.
          GameSettingsStore::new(settings.clone(), directories.get_settings_files()),
          flow,
        ),
        layers,
      );
      let played: XrfResult<()> = play.run(&mut shell.platform);

      // However play ended, what was changed while it ran is kept for the next run, as X-Ray saves its console's on
      // exit.
      Self::save_settings(&mut play.get_context_mut().settings);

      played
    };

    shell.stop();

    played
  }

  /// Saves the settings changed since the last save, logging what it did; nothing where none changed.
  fn save_settings(store: &mut GameSettingsStore) {
    let count: usize = store.count_unsaved();

    match store.save() {
      Ok(Some(file)) => log::info!("Saved {count} changed settings to '{}'", file.display()),
      Ok(None) => {}
      Err(error) => log::error!("The settings were not saved: {error}"),
    }
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
