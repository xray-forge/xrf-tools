use std::io::ErrorKind;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use rayon::ThreadPool;
use xrf_error::{XrfError, XrfResult};
use xrf_game_loop::{GameClock, GameLoop, LoopClock, LoopStepper};
use xrf_game_movement::{FlyMode, Movement, MovementMode};
use xrf_game_platform::{
  Platform, PlatformSize, PlatformWindow, PlatformWindowCommand, PlatformWindowHandle, PlatformWindowMode,
  PlatformWindowRequest,
};
use xrf_game_settings::{GameSettings, GameWindowMode};
use xrf_job::ExecutionRequest;
use xrf_level_source::{LevelAudio, LevelRenderSource};
use xrf_renderer::RenderWorkers;
use xrf_renderer_host::HostedWindow;
use xrf_world::{WorldFedCamera, WorldLevelSource};

use crate::arguments::game_arguments::GameArguments;
use crate::arguments::game_mode_argument::GameModeArgument;
#[cfg(feature = "mcp")]
use crate::control::game_control::GameControl;
use crate::directories::game_directories::GameDirectories;
use crate::input::game_bindings::GameBindings;
use crate::level::game_level::GameLevel;
use crate::level::game_tree::GameTree;
use crate::render::game_view::GameView;
use crate::run::game_play::GamePlay;
use crate::sound::game_sound::GameSound;
use crate::state::game_state::GameState;
use crate::state::game_view_feed::GameViewFeed;

/// One run of the game: the tree and level its arguments name opened, the window and its view started, the game loop
/// stepping the player's movement, and the platform's events read into the loop's commands until the player quits.
pub struct GameRun;

impl GameRun {
  /// Plays the level `arguments` name until the player or an agent quits.
  ///
  /// # Errors
  ///
  /// Returns why the game cannot start: no tree or level named (with the levels the tree has), a mode or endpoint not
  /// built, a tree or level that does not read, or a window, renderer or endpoint that cannot start.
  pub fn run(arguments: &GameArguments, settings: &GameSettings, directories: &GameDirectories) -> XrfResult<()> {
    if cfg!(not(feature = "mcp")) && arguments.mcp {
      return Err(XrfError::new_not_implemented_error(
        "this build has no control endpoint; build with the mcp feature",
      ));
    }

    if arguments.mode == GameModeArgument::Walk {
      return Err(XrfError::new_not_implemented_error(
        "walk mode comes with the physics engine; start with --mode fly",
      ));
    }

    let Some(root) = &arguments.root else {
      return Err(XrfError::new_invalid_error(
        "name the game tree to play with --root <path> and its level with --level <name>",
      ));
    };
    let tree: GameTree = Self::mount(root)?;
    let Some(name) = &arguments.level else {
      return Err(XrfError::new_invalid_error(format!(
        "name the level to play with --level <name>: the tree has {}",
        tree.list_levels()?.join(", ")
      )));
    };
    let pool: Arc<ThreadPool> = Arc::new(ExecutionRequest::Auto.resolve().build_pool()?);
    let started: Instant = Instant::now();
    let level: GameLevel = GameLevel::open(&tree, name, &pool)?;
    let audio: LevelAudio = GameSound::start()?;
    let source: Arc<dyn WorldLevelSource> = Arc::new(
      pool.install(|| LevelRenderSource::new(Arc::clone(level.get_snapshot()), Arc::clone(tree.get_host()), audio)),
    );

    log::info!("Level '{name}' opened in {:.1?}", started.elapsed());

    let mut platform: Platform = Platform::start().map_err(Self::create_platform_error)?;
    let mut window: PlatformWindow = platform
      .open_window(&PlatformWindowRequest {
        title: format!("XRF: {name}"),
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
    let state: GameState = GameState {
      movement: Movement::new(level.get_start_camera(), MovementMode::Fly(FlyMode::default())),
    };
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
        .then(|| GameControl::start(name, directories.get_captures_directory()))
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

  fn mount(root: &Path) -> XrfResult<GameTree> {
    let tree: GameTree = GameTree::mount(root)?;

    log::info!(
      "Game tree {} mounted, engine {:?}",
      root.display(),
      tree.get_engine().engine
    );

    Ok(tree)
  }

  fn create_platform_error(message: String) -> XrfError {
    XrfError::new_io_error(message, ErrorKind::Other)
  }
}
