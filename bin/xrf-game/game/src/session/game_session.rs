use std::sync::Arc;
use std::time::Instant;

use rayon::ThreadPool;
use xrf_error::{XrfError, XrfResult};
use xrf_game_alife::{Alife, AlifeData, AlifeSettings};
use xrf_game_movement::{FlyMode, Movement, MovementMode};
use xrf_game_objects::ObjectRegistry;
use xrf_game_simulation::{SimulationConfig, SmartSpawn};
use xrf_level_source::{LevelAudio, LevelRenderSource};
use xrf_world::WorldLevelSource;

use crate::arguments::game_arguments::GameArguments;
use crate::arguments::game_mode_argument::GameModeArgument;
use crate::level::game_configs::GameConfigs;
use crate::level::game_level::GameLevel;
use crate::level::game_spawn::GameSpawn;
use crate::level::game_tree::GameTree;
use crate::session::game_start::GameStart;
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
    // The level and the new game read different files and share nothing until the state is put together.
    let (level, start): (XrfResult<GameLevel>, XrfResult<GameStart>) = pool.join(
      || GameLevel::open(&tree, &level_name, pool),
      || Self::read_start(&tree, &level_name),
    );
    let level: GameLevel = level?;
    let start: GameStart = start?;
    let audio: LevelAudio = GameSound::start()?;
    let source: Arc<dyn WorldLevelSource> = Arc::new(
      pool.install(|| LevelRenderSource::new(Arc::clone(level.get_snapshot()), Arc::clone(tree.get_host()), audio)),
    );
    let state: GameState = GameState {
      objects: start.objects,
      movement: Movement::new(level.get_start_camera(), MovementMode::Fly(FlyMode::default())),
      alife: start.alife,
    };

    log::info!("Level '{level_name}' opened in {:.1?}", started.elapsed());

    Ok(Self {
      level_name,
      source,
      state,
    })
  }

  /// How many lines of issues the simulation's configs log before counting the rest.
  const LOGGED_ISSUES: usize = 10;

  /// What a new game starts with beside the player: the spawn's objects, and A-Life by the tree's `[alife]` over the
  /// spawn's graph and the simulation's configs, the actor on the level named `level_name`.
  fn read_start(tree: &GameTree, level_name: &str) -> XrfResult<GameStart> {
    let started: Instant = Instant::now();
    let spawn: GameSpawn = tree.read_spawn()?;
    let objects: ObjectRegistry = ObjectRegistry::of_spawn(&spawn.placed);
    let level: Option<u8> = spawn.graph.find_level(level_name).map(|level| level.id);
    let on_level: usize = level.map_or(0, |level| objects.count_on_level(level));

    log::info!(
      "New game holds {} of the spawn's {} objects, {on_level} on '{level_name}', {} skipped, on a graph of {} vertices \
       and {} edges, read in {:.1?}",
      objects.get_records().len(),
      spawn.placed.total,
      spawn.placed.skipped.len(),
      spawn.graph.len(),
      spawn.graph.count_edges(),
      started.elapsed()
    );

    for skipped in spawn.placed.skipped.iter().take(5) {
      log::warn!("Spawn object {} skipped: {}", skipped.index, skipped.reason);
    }

    let opened: Instant = Instant::now();
    let configs: GameConfigs = tree.open_configs()?;

    log::info!("Configs opened, system.ltx resolved, in {:.1?}", opened.elapsed());

    let settings: AlifeSettings = configs.read_alife_settings();

    log::info!(
      "A-Life switches objects online within {} m and offline beyond {} m, every {:?}",
      settings.get_online_distance(),
      settings.get_offline_distance(),
      settings.get_update_interval()
    );

    Ok(GameStart {
      objects,
      alife: Alife::new(settings, Self::read_alife_data(&configs, spawn), level),
    })
  }

  /// What A-Life reads: the spawn's graph and the simulation's configs, read from `configs` for the smart terrains the
  /// spawn places.
  fn read_alife_data(configs: &GameConfigs, spawn: GameSpawn) -> AlifeData {
    let started: Instant = Instant::now();
    let levels: Vec<String> = spawn
      .graph
      .get_levels()
      .iter()
      .map(|level| level.name.clone())
      .collect();
    let simulation: SimulationConfig = SimulationConfig::read(configs, &levels, &SmartSpawn::of_spawn(&spawn.placed));

    log::info!(
      "Simulation: {} smart terrains, {} squad sections, {} start squads, {} behaviours, {} props, {} issues, read in \
       {:.1?}",
      simulation.get_smarts().len(),
      simulation.get_squads().len(),
      simulation.get_start().len(),
      simulation.count_behaviours(),
      simulation.count_props(),
      simulation.get_issues().len(),
      started.elapsed()
    );

    for issue in simulation.get_issues().iter().take(Self::LOGGED_ISSUES) {
      log::warn!("Simulation: {issue}");
    }

    if simulation.get_issues().len() > Self::LOGGED_ISSUES {
      log::warn!(
        "Simulation: {} more issues",
        simulation.get_issues().len() - Self::LOGGED_ISSUES
      );
    }

    AlifeData {
      graph: Arc::new(spawn.graph),
      simulation: Arc::new(simulation),
    }
  }
}
