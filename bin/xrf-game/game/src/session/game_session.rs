use std::sync::Arc;
use std::time::Instant;

use rayon::ThreadPool;
use xrf_error::{XrfError, XrfResult};
use xrf_game_movement::{FlyMode, Movement, MovementMode};
use xrf_game_objects::ObjectRegistry;
use xrf_level_source::{LevelAudio, LevelRenderSource};
use xrf_spawn::SpawnPlacedObjects;
use xrf_world::WorldLevelSource;

use crate::arguments::game_arguments::GameArguments;
use crate::arguments::game_mode_argument::GameModeArgument;
use crate::level::game_level::GameLevel;
use crate::level::game_tree::GameTree;
use crate::sound::game_sound::GameSound;
use crate::state::game_state::GameState;

/// What one run plays, opened from the game tree before any window: the level as the world shows it, and a new game's
/// state (every object of the spawn, the player's movement at the level's start). It knows nothing of the window, the
/// renderer's thread or the loop that will step the state.
pub struct GameSession {
  /// The level's name, as the tree names it.
  pub level_name: String,
  /// The level as the world shows it.
  pub source: Arc<dyn WorldLevelSource>,
  /// A new game's state, which the loop takes.
  pub state: GameState,
}

impl GameSession {
  /// Opens the tree and level `arguments` name and builds a new game on it, reading on `pool`.
  ///
  /// # Errors
  ///
  /// Returns why it cannot: no tree or level named (with the levels the tree has), a mode not built yet, or a tree,
  /// level or spawn that does not read.
  pub fn open(arguments: &GameArguments, pool: &ThreadPool) -> XrfResult<Self> {
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
    let tree: GameTree = GameTree::mount(root)?;

    log::info!(
      "Game tree {} mounted, engine {:?}",
      root.display(),
      tree.get_engine().engine
    );

    let Some(level_name) = arguments.level.clone() else {
      return Err(XrfError::new_invalid_error(format!(
        "name the level to play with --level <name>: the tree has {}",
        tree.list_levels()?.join(", ")
      )));
    };
    let started: Instant = Instant::now();
    let level: GameLevel = GameLevel::open(&tree, &level_name, pool)?;
    let objects: ObjectRegistry = Self::read_objects(&tree, &level_name)?;
    let audio: LevelAudio = GameSound::start()?;
    let source: Arc<dyn WorldLevelSource> = Arc::new(
      pool.install(|| LevelRenderSource::new(Arc::clone(level.get_snapshot()), Arc::clone(tree.get_host()), audio)),
    );
    let state: GameState = GameState {
      objects,
      movement: Movement::new(level.get_start_camera(), MovementMode::Fly(FlyMode::default())),
    };

    log::info!("Level '{level_name}' opened in {:.1?}", started.elapsed());

    Ok(Self {
      level_name,
      source,
      state,
    })
  }

  /// A new game's objects: every object the tree's spawn places.
  fn read_objects(tree: &GameTree, level_name: &str) -> XrfResult<ObjectRegistry> {
    let started: Instant = Instant::now();
    let placed: SpawnPlacedObjects = tree.read_spawn()?;
    let objects: ObjectRegistry = ObjectRegistry::of_spawn(&placed);
    let on_level: usize = objects
      .find_level(level_name)
      .map_or(0, |level| objects.count_on_level(level));

    log::info!(
      "New game holds {} of the spawn's {} objects, {on_level} on '{level_name}', {} skipped, read in {:.1?}",
      objects.get_records().len(),
      placed.total,
      placed.skipped.len(),
      started.elapsed()
    );

    for skipped in placed.skipped.iter().take(5) {
      log::warn!("Spawn object {} skipped: {}", skipped.index, skipped.reason);
    }

    Ok(objects)
  }
}
