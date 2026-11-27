use std::fmt::Display;
use std::sync::Arc;
use std::time::Instant;

use rayon::ThreadPool;
use xrf_error::{XrfError, XrfResult};
use xrf_game_alife::{Alife, AlifeData, AlifeHooks};
use xrf_game_config::ConfigIssues;
use xrf_game_hooks::HookRegistry;
use xrf_game_loop::LoopSession;
use xrf_game_movement::{FlyMode, Movement, MovementModes};
use xrf_game_objects::{ObjectId, ObjectRecord, ObjectRegistry, ObjectSwitching};
use xrf_game_simulation::{SimulationConfig, SimulationStart, SimulationStartReport, SmartSpawn};
use xrf_gamemtl::GameMtlFile;
use xrf_level::LevelCformGeometry;
use xrf_level_source::{LevelAudio, LevelRenderSource};
use xrf_math::glam::Vec3;
use xrf_physics::{PhysicsLevel, PhysicsWorld};
use xrf_spawn::ClsId;
use xrf_world::WorldLevelSource;

use crate::host::game_alife_config::GameAlifeConfig;
use crate::host::game_configs::GameConfigs;
use crate::host::game_host::GameHost;
use crate::host::game_player_config::GamePlayerConfig;
use crate::host::game_tree::GameTree;
use crate::input::game_controls::GameControls;
use crate::level::game_level::GameLevel;
use crate::load::game_load_progress::GameLoadProgress;
use crate::load::game_load_stage::GameLoadStage;
use crate::record::game_recorder::GameRecorder;
use crate::record::game_recording_header::GameRecordingHeader;
use crate::record::game_replay::GameReplay;
use crate::session::game_load_request::GameLoadRequest;
use crate::session::game_load_source::GameLoadSource;
use crate::session::game_recording_request::GameRecordingRequest;
use crate::session::game_spawn::GameSpawn;
use crate::session::game_start::GameStart;
use crate::sound::game_footsteps::GameFootsteps;
use crate::sound::game_sound::GameSound;
use crate::state::game_level_state::GameLevelState;
use crate::state::game_progress::GameProgress;
use crate::state::game_state::GameState;
use crate::time::game_time::GameTime;

/// What a load opens from its request, before the loop steps anything: the host the run keeps (the tree, its configs
/// and materials, the audio server), the level as the world shows it, and the game's state on it. It knows nothing of
/// the window, the renderer's thread or the loop that will step the state.
pub struct GameSession {
  pub host: GameHost,
  /// The level as the world shows it.
  pub source: Arc<dyn WorldLevelSource>,
  /// The game's state on the level, which the loop takes.
  pub state: GameState,
  /// The player's footsteps, which presentation plays from the loop's frames.
  pub footsteps: GameFootsteps,
  /// The tree's own controls, which presentation lays the player's bindings over.
  pub controls: GameControls,
  /// Where the loop over the state takes its commands and who watches it step: live and unwatched, unless the
  /// request records the session or replays one.
  pub loop_session: LoopSession<GameState>,
}

impl GameSession {
  /// How many lines of issues the simulation's configs log before counting the rest.
  const LOGGED_ISSUES: usize = 10;

  /// Opens what `request` asks on `tree`, mounted already: the level, the game on it and the level's physics, read at
  /// once on `pool`, each stage reported to `progress` as it begins and ends.
  ///
  /// # Errors
  ///
  /// Returns why it cannot: a level the tree does not hold (with the levels it holds), or a level, spawn, config or
  /// collision that does not read.
  pub(crate) fn open(
    request: &GameLoadRequest,
    tree: GameTree,
    pool: &ThreadPool,
    progress: &GameLoadProgress,
  ) -> XrfResult<Self> {
    let GameLoadSource::New { seed } = request.game;
    let level_name: &str = &request.level;
    let started: Instant = Instant::now();
    // First, so a recording that cannot be written or does not read stops the load before the level is read.
    let loop_session: LoopSession<GameState> = Self::open_loop_session(request)?;

    log::info!("New game seed {seed}; --seed {seed} starts the same game");

    // The level's physics and the player's footsteps both read the game materials.
    let materials: GameMtlFile = tree.read_materials()?;
    // The level, the new game and the level's physics read different files and share nothing until the state is put
    // together, so each reads on the pool at once.
    let mut level: Option<XrfResult<GameLevel>> = None;
    let mut start: Option<XrfResult<GameStart>> = None;
    let mut physics: Option<XrfResult<PhysicsWorld>> = None;

    pool.scope(|scope| {
      scope.spawn(|_| level = Some(progress.run(GameLoadStage::Level, || GameLevel::open(&tree, level_name, pool))));
      scope.spawn(|_| start = Some(progress.run(GameLoadStage::Game, || Self::read_start(&tree, level_name, seed))));
      scope.spawn(|_| {
        physics = Some(progress.run(GameLoadStage::Physics, || {
          Self::read_physics(&tree, level_name, &materials)
        }));
      });
    });

    let unread = || XrfError::new_unexpected_error("a part of the level was not read");
    let level: GameLevel = level.ok_or_else(unread)??;
    let start: GameStart = start.ok_or_else(unread)??;
    let physics: PhysicsWorld = physics.ok_or_else(unread)??;
    let audio: LevelAudio = GameSound::start()?;
    let source: Arc<dyn WorldLevelSource> = Arc::new(pool.install(|| {
      LevelRenderSource::new(
        Arc::clone(level.get_snapshot()),
        Arc::clone(tree.get_host()),
        audio.clone(),
      )
    }));
    let footsteps: GameFootsteps = GameFootsteps::of_materials(&materials, start.player_config.step_volume, seed);
    let state: GameState = GameState {
      progress: GameProgress {
        seed,
        time: GameTime::new(start.clock),
        objects: start.objects,
        alife: start.alife,
        player: start.player,
      },
      level: GameLevelState {
        physics,
        graph_level: start.graph_level,
      },
      hooks: start.hooks,
      movement: Movement::new(
        level.get_start_camera(),
        request.mode,
        MovementModes {
          fly: FlyMode::default(),
          walk: start.player_config.walk,
        },
      ),
    };

    log::info!("Level '{level_name}' opened in {:.1?}", started.elapsed());

    Ok(Self {
      host: GameHost::new(tree, start.configs, materials, audio),
      source,
      state,
      footsteps,
      controls: start.controls,
      loop_session,
    })
  }

  /// The loop session `request` asks for: one recorded to its file, one stepping its recording again with each digest
  /// checked, or a live one nothing watches.
  fn open_loop_session(request: &GameLoadRequest) -> XrfResult<LoopSession<GameState>> {
    Ok(match &request.recording {
      None => LoopSession::default(),
      Some(GameRecordingRequest::Record(file)) => LoopSession {
        replay: None,
        witness: Some(Box::new(GameRecorder::create(file, &GameRecordingHeader::of(request))?)),
      },
      Some(GameRecordingRequest::Replay(file)) => GameReplay::read(file)?.into_session(),
    })
  }

  /// Logs up to [`Self::LOGGED_ISSUES`] of `issues` under `subject`, and how many more there are.
  fn log_issues<T: Display>(subject: &str, issues: impl ExactSizeIterator<Item = T>) {
    let count: usize = issues.len();

    for issue in issues.take(Self::LOGGED_ISSUES) {
      log::warn!("{subject}: {issue}");
    }

    if count > Self::LOGGED_ISSUES {
      log::warn!("{subject}: {} more issues", count - Self::LOGGED_ISSUES);
    }
  }

  /// What a new game starts with beside the player's movement: the spawn's objects and the simulation's start squads,
  /// the player's record among them, and A-Life by the tree's `[alife]` over the spawn's graph and the simulation's
  /// configs, the game on the level named `level_name`, every draw keyed by `seed`.
  fn read_start(tree: &GameTree, level_name: &str, seed: u64) -> XrfResult<GameStart> {
    let started: Instant = Instant::now();
    let spawn: GameSpawn = tree.read_spawn()?;
    let mut objects: ObjectRegistry = ObjectRegistry::of_spawn(&spawn.placed);
    let graph_level: Option<u8> = spawn.graph.find_level(level_name).map(|level| level.id);
    let on_level: usize = graph_level.map_or(0, |level| objects.count_on_level(level));

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

    let player: ObjectId = Self::find_player(&mut objects)?;
    let opened: Instant = Instant::now();
    let configs: GameConfigs = tree.open_configs()?;

    log::info!("Configs opened, system.ltx resolved, in {:.1?}", opened.elapsed());

    let mut issues: ConfigIssues = ConfigIssues::default();
    let GameAlifeConfig { settings, clock } = configs.read_alife(&mut issues);
    let player_config: GamePlayerConfig = configs.read_player_config(&mut issues);
    let controls: GameControls = configs.read_controls(&mut issues)?;

    Self::log_issues("Config", issues.iter());

    log::info!(
      "A-Life switches objects online within {} m and offline beyond {} m, every {:?}; the day starts at {:.0} s \
       past midnight, {} game seconds a second",
      settings.get_online_distance(),
      settings.get_offline_distance(),
      settings.get_update_interval(),
      clock.start_time,
      clock.factor
    );

    let data: AlifeData = Self::read_alife_data(&configs, spawn);
    // Every hook point the game's systems declare, each holding its system's own behaviour until a script replaces it.
    let mut hooks: HookRegistry = HookRegistry::default();

    AlifeHooks::register(&mut hooks)?;

    let created: Instant = Instant::now();
    let report: SimulationStartReport = SimulationStart {
      config: &data.simulation,
      graph: &data.graph,
      patrols: &data.patrols,
      hooks: &hooks,
      seed,
    }
    .create(&mut objects);
    let on_level: usize = graph_level.map_or(0, |level| {
      objects
        .get_groups()
        .iter()
        .filter(|(id, _)| objects.get_level(*id) == Some(level))
        .count()
    });

    log::info!(
      "New game starts {} squads of {} members, {on_level} squads on '{level_name}', {} issues, in {:.1?}",
      report.squads,
      report.members,
      report.issues.len(),
      created.elapsed()
    );
    Self::log_issues("Start", report.issues.iter());

    Ok(GameStart {
      objects,
      player,
      alife: Alife::new(settings, data),
      clock,
      hooks,
      graph_level,
      player_config,
      controls,
      configs,
    })
  }

  /// The player's object: the spawn's actor, as X-Ray's new game spawns it, or one made where the spawn holds none.
  ///
  /// # Errors
  ///
  /// Returns why no actor can be made: the object id space is spent.
  fn find_player(objects: &mut ObjectRegistry) -> XrfResult<ObjectId> {
    if let Some((id, _)) = objects
      .get_records()
      .iter()
      .find(|(_, record)| record.class == ClsId::SActor)
    {
      return Ok(id);
    }

    log::warn!("The spawn holds no actor; the game makes one for the player");

    objects
      .create(
        ObjectRecord {
          section: String::from("actor"),
          name: String::from("actor"),
          class: ClsId::SActor,
          position: Vec3::ZERO,
          direction: Vec3::ZERO,
          game_vertex: None,
          level_vertex: None,
          story_id: None,
          spawn_id: u16::MAX,
          switching: ObjectSwitching::ByDistance,
        },
        None,
      )
      .ok_or_else(|| XrfError::new_unexpected_error("no object id is left for the player"))
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

  /// What A-Life reads: the spawn's graph and patrols, and the simulation's configs, read from `configs` for the smart
  /// terrains the spawn places.
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

    Self::log_issues("Simulation", simulation.get_issues().iter());

    AlifeData {
      graph: Arc::new(spawn.graph),
      patrols: Arc::new(spawn.patrols),
      simulation: Arc::new(simulation),
    }
  }
}
