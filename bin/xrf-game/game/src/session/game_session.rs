use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use rayon::ThreadPool;
use xrf_error::{XrfError, XrfResult};
use xrf_game_alife::{Alife, AlifeData, AlifeSettings};
use xrf_game_movement::{FlyMode, Movement, MovementMode, MovementModes};
use xrf_game_objects::ObjectRegistry;
use xrf_game_simulation::{SimulationConfig, SimulationStart, SimulationStartReport, SmartSpawn};
use xrf_gamemtl::GameMtlFile;
use xrf_level::LevelCformGeometry;
use xrf_level_source::{LevelAudio, LevelRenderSource};
use xrf_physics::{PhysicsLevel, PhysicsWorld};
use xrf_world::WorldLevelSource;

use crate::arguments::game_arguments::GameArguments;
use crate::arguments::game_mode_argument::GameModeArgument;
use crate::level::game_configs::GameConfigs;
use crate::level::game_level::GameLevel;
use crate::level::game_spawn::GameSpawn;
use crate::level::game_tree::GameTree;
use crate::session::game_start::GameStart;
use crate::sound::game_footsteps::GameFootsteps;
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
  /// The player's footsteps, which presentation plays from the loop's frames.
  pub footsteps: GameFootsteps,
}

impl GameSession {
  /// Opens the tree and level `arguments` name and builds a new game on it, reading on `pool`.
  ///
  /// # Errors
  ///
  /// Returns why it cannot: no tree or level named (with the levels the tree has), a mode not built yet, or a tree,
  /// level or spawn that does not read.
  pub fn open(arguments: &GameArguments, pool: &ThreadPool) -> XrfResult<Self> {
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
    let seed: u64 = arguments.seed.unwrap_or_else(Self::create_seed);

    log::info!("New game seed {seed}; --seed {seed} starts the same game");

    // The level's physics and the player's footsteps both read the game materials.
    let materials: GameMtlFile = tree.read_materials()?;
    // The level, the new game and the level's physics read different files and share nothing until the state is put
    // together.
    let (level, (start, physics)): (XrfResult<GameLevel>, (XrfResult<GameStart>, XrfResult<PhysicsWorld>)) = pool.join(
      || GameLevel::open(&tree, &level_name, pool),
      || {
        pool.join(
          || Self::read_start(&tree, &level_name, seed),
          || Self::read_physics(&tree, &level_name, &materials),
        )
      },
    );
    let level: GameLevel = level?;
    let start: GameStart = start?;
    let physics: PhysicsWorld = physics?;
    let mode: MovementMode = match arguments.mode {
      GameModeArgument::Fly => MovementMode::Fly,
      GameModeArgument::Walk => MovementMode::Walk,
    };
    let modes: MovementModes = MovementModes {
      fly: FlyMode::default(),
      walk: start.player.walk,
    };
    let audio: LevelAudio = GameSound::start()?;
    let source: Arc<dyn WorldLevelSource> = Arc::new(
      pool.install(|| LevelRenderSource::new(Arc::clone(level.get_snapshot()), Arc::clone(tree.get_host()), audio)),
    );
    let state: GameState = GameState {
      objects: start.objects,
      physics,
      movement: Movement::new(level.get_start_camera(), mode, modes),
      alife: start.alife,
    };
    let footsteps: GameFootsteps = GameFootsteps::of_materials(&materials, start.player.step_volume, seed);

    log::info!("Level '{level_name}' opened in {:.1?}", started.elapsed());

    Ok(Self {
      level_name,
      source,
      state,
      footsteps,
    })
  }

  /// How many lines of issues the simulation's configs log before counting the rest.
  const LOGGED_ISSUES: usize = 10;

  /// A seed for a run that names none: the wall clock's nanoseconds, the one place a new game reads it.
  fn create_seed() -> u64 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map_or(0, |since| since.as_nanos() as u64)
  }

  /// Logs up to [`Self::LOGGED_ISSUES`] of `issues` under `subject`, and how many more there are.
  fn log_issues(subject: &str, issues: &[String]) {
    for issue in issues.iter().take(Self::LOGGED_ISSUES) {
      log::warn!("{subject}: {issue}");
    }

    if issues.len() > Self::LOGGED_ISSUES {
      log::warn!("{subject}: {} more issues", issues.len() - Self::LOGGED_ISSUES);
    }
  }

  /// What a new game starts with beside the player: the spawn's objects and the simulation's start squads, and A-Life
  /// by the tree's `[alife]` over the spawn's graph and the simulation's configs, the actor on the level named
  /// `level_name`, every draw keyed by `seed`.
  fn read_start(tree: &GameTree, level_name: &str, seed: u64) -> XrfResult<GameStart> {
    let started: Instant = Instant::now();
    let spawn: GameSpawn = tree.read_spawn()?;
    let mut objects: ObjectRegistry = ObjectRegistry::of_spawn(&spawn.placed);
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

    let data: AlifeData = Self::read_alife_data(&configs, spawn, seed);
    let created: Instant = Instant::now();
    let report: SimulationStartReport = SimulationStart {
      config: &data.simulation,
      graph: &data.graph,
      patrols: &data.patrols,
      seed,
    }
    .create(&mut objects);
    let on_level: usize = level.map_or(0, |level| {
      objects
        .get_groups()
        .iter()
        .filter(|(id, _)| {
          objects
            .get_records()
            .get(*id)
            .is_some_and(|record| record.level == Some(level))
        })
        .count()
    });

    log::info!(
      "New game starts {} squads of {} members, {on_level} squads on '{level_name}', {} issues, in {:.1?}",
      report.squads,
      report.members,
      report.issues.len(),
      created.elapsed()
    );
    Self::log_issues("Start", &report.issues);

    Ok(GameStart {
      objects,
      alife: Alife::new(settings, data, level),
      player: configs.read_player_config(),
    })
  }

  /// The physics world of the level `level_name`: its collision form, a mesh for each of its game `materials`, which
  /// takes a second or two for a large level, so it is built beside the rest of the level.
  fn read_physics(tree: &GameTree, level_name: &str, materials: &GameMtlFile) -> XrfResult<PhysicsWorld> {
    let started: Instant = Instant::now();
    let collision: LevelCformGeometry = tree.read_collision(level_name)?;
    let level: PhysicsLevel = PhysicsLevel::of_cform(&collision, materials)?;
    let world: PhysicsWorld = PhysicsWorld::with_level(&level)?;

    log::info!(
      "Physics: '{level_name}' has {} faces, {} passable left out, {} too thin dropped, built in {:.1?}",
      collision.get_faces().len(),
      level.get_passable(),
      level.get_dropped(),
      started.elapsed()
    );

    Ok(world)
  }

  /// What A-Life reads: the spawn's graph and patrols, the simulation's configs, read from `configs` for the smart
  /// terrains the spawn places, and the game's `seed`.
  fn read_alife_data(configs: &GameConfigs, spawn: GameSpawn, seed: u64) -> AlifeData {
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

    Self::log_issues("Simulation", simulation.get_issues());

    AlifeData {
      graph: Arc::new(spawn.graph),
      patrols: Arc::new(spawn.patrols),
      simulation: Arc::new(simulation),
      seed,
    }
  }
}
