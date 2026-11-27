use std::io::ErrorKind;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use rayon::ThreadPool;
use xrf_error::{XrfError, XrfResult};
use xrf_game_loop::{GameClock, GameLoop, LoopClock, LoopStepper};
use xrf_game_movement::{FlyMode, Movement, MovementMode};
use xrf_game_platform::{
  Platform, PlatformEvent, PlatformEventKind, PlatformScancode, PlatformSize, PlatformWindow, PlatformWindowCommand,
  PlatformWindowHandle, PlatformWindowMode, PlatformWindowRequest,
};
use xrf_game_settings::{GameSettings, GameWindowMode};
use xrf_job::ExecutionRequest;
use xrf_level_source::{LevelAudio, LevelRenderSource};
use xrf_renderer::RenderWorkers;
use xrf_renderer_host::HostedWindow;
use xrf_world::{WorldFedCamera, WorldLevelSource};

use crate::arguments::game_arguments::GameArguments;
use crate::arguments::game_mode_argument::GameModeArgument;
use crate::directories::game_directories::GameDirectories;
use crate::input::game_bindings::GameBindings;
use crate::level::game_level::GameLevel;
use crate::level::game_tree::GameTree;
use crate::render::game_view::GameView;
use crate::sound::game_sound::GameSound;
use crate::state::game_command::GameCommand;
use crate::state::game_state::GameState;
use crate::state::game_view_feed::GameViewFeed;

/// One run of the game: the tree and level its arguments name opened, the window and its view started, the game loop
/// stepping the player's movement, and the platform's events read into the loop's commands until the player quits.
pub struct GameRun;

impl GameRun {
  /// The key that quits until the game has a menu: Escape's place.
  const QUIT_KEY: PlatformScancode = PlatformScancode(41);

  /// How long the main thread waits between reads of the platform's events.
  const POLL_WAIT: Duration = Duration::from_millis(1);

  /// Plays the level `arguments` name until the player quits.
  ///
  /// # Errors
  ///
  /// Returns why the game cannot start: no tree or level named (with the levels the tree has), a mode not built yet, a
  /// tree or level that does not read, or a window or renderer that cannot start.
  pub fn run(arguments: &GameArguments, settings: &GameSettings, directories: &GameDirectories) -> XrfResult<()> {
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

    Self::play(&mut platform, &window, &view, &game_loop.get_sender());

    // Stopped in the order the window's drop needs: the loop, the view, then the window.
    drop(game_loop);
    view.stop();
    drop(hosted);
    drop(window);

    Ok(())
  }

  /// Reads the platform's events into the loop's commands until the player quits, fitting the view as the window is
  /// resized.
  fn play(platform: &mut Platform, window: &PlatformWindow, view: &GameView, commands: &Sender<GameCommand>) {
    let bindings: GameBindings = GameBindings::default();
    let mut events: Vec<PlatformEvent> = Vec::new();

    loop {
      events.clear();
      platform.poll(&mut events);

      for event in &events {
        match &event.kind {
          PlatformEventKind::Quit
          | PlatformEventKind::Key {
            scancode: Self::QUIT_KEY,
            is_down: true,
            ..
          } => return,
          PlatformEventKind::Resized { .. } => view.fit(window),
          _ => {
            if let Some(command) = bindings.read(event)
              && commands.send(command).is_err()
            {
              log::error!("The game loop stopped taking commands");

              return;
            }
          }
        }
      }

      std::thread::sleep(Self::POLL_WAIT);
    }
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
