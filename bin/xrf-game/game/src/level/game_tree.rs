use std::path::Path;
use std::sync::Arc;

use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};
use xrf_dltx::DltxDialect;
use xrf_engine_target::{XrayEngine, XrayEngineChoice, XrayEngineDetector, XrayEngineResolution};
use xrf_error::XrfResult;
use xrf_game_graph::{GameGraph, GamePatrols};
use xrf_gamemtl::GameMtlFile;
use xrf_level::{LevelCformFile, LevelCformGeometry};
use xrf_level_source::{LevelEntry, LevelSource};
use xrf_ltx::LtxDialect;
use xrf_ltx_project::{LtxProject, LtxProjectOptions};
use xrf_spawn::{SpawnFile, SpawnPatrolsChunk};
use xrf_vfs::{XrayAssetType, XrayCachePolicy, XrayMountMode, XrayRoots, XraySearch, XrayVfs, XrayVfsHost};

use crate::level::game_configs::GameConfigs;
use crate::level::game_spawn::GameSpawn;

/// The X-Ray game tree the game plays: its roots mounted as the tools mount them (an installation with its archives,
/// a set of volumes, or loose files), the file system host every read goes through, and the engine it was made for.
pub struct GameTree {
  roots: XrayRoots,
  host: Arc<XrayVfsHost>,
  engine: XrayEngineResolution,
}

impl GameTree {
  /// The most the host's cache keeps of the files it caches, as the desktop app's does.
  const CACHE_BUDGET: u64 = 64 * 1024 * 1024;

  /// The game's spawn, which places what a new game starts with on every level.
  const SPAWN_FILE: &'static str = "spawns\\all.spawn";

  /// Where the tree keeps its configs.
  const CONFIGS_DIRECTORY: &'static str = "configs";

  /// Mounts the tree at `root`, its kind and its engine detected.
  ///
  /// # Errors
  ///
  /// Returns why the tree cannot be mounted or searched.
  pub fn mount(root: &Path) -> XrfResult<Self> {
    let roots: XrayRoots = XrayRoots::one(root.to_path_buf(), XrayMountMode::Auto);
    let policy: XrayCachePolicy = XrayCachePolicy::configs()
      .with(XrayAssetType::Ogf)
      .with(XrayAssetType::Thm)
      .with_budget(Self::CACHE_BUDGET);
    let host: Arc<XrayVfsHost> = Arc::new(XrayVfsHost::new(XrayVfs::new().with_cache_policy(policy)));
    let search: XraySearch = host.search(&roots)?;
    let engine: XrayEngineResolution = XrayEngineChoice::Auto
      .resolve(|| XrayEngineDetector::detect(&roots, |logical_path| search.read(logical_path).ok()));

    Ok(Self { roots, host, engine })
  }

  pub fn get_roots(&self) -> &XrayRoots {
    &self.roots
  }

  pub fn get_host(&self) -> &Arc<XrayVfsHost> {
    &self.host
  }

  pub fn get_engine(&self) -> &XrayEngineResolution {
    &self.engine
  }

  /// A search of the tree as it stands.
  ///
  /// # Errors
  ///
  /// Returns why the tree cannot be searched.
  pub fn search(&self) -> XrfResult<XraySearch> {
    self.host.search(&self.roots)
  }

  /// How its configs are read: DLTX for an extended engine (Anomaly), which its mods write, else the standard dialect.
  pub fn get_dialect(&self) -> Arc<dyn LtxDialect> {
    DltxDialect::select(self.engine.engine == XrayEngine::Extended)
  }

  /// What a new game starts from, read from its spawn (`spawns\all.spawn`) in one pass: every object it places, each
  /// with its level, those no reader takes skipped, and the game graph.
  ///
  /// # Errors
  ///
  /// Returns why the spawn cannot be read: the tree has none, or its chunks or graph do not read.
  pub fn read_spawn(&self) -> XrfResult<GameSpawn> {
    let bytes: Vec<u8> = self.search()?.read(Self::SPAWN_FILE)?;
    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(bytes)?.read_children()?;

    Ok(GameSpawn {
      placed: SpawnFile::read_objects_from_chunks(&chunks)?,
      graph: GameGraph::of_spawn(&SpawnFile::read_graphs_from_chunks(&chunks)?)?,
      patrols: GamePatrols::of_spawn(
        &ChunkReader::find_required_by_id(&chunks, SpawnPatrolsChunk::CHUNK_ID)?.read_xr::<SpawnPatrolsChunk>()?,
      ),
    })
  }

  /// Its configs (`configs`), read in its dialect, `system.ltx` resolved.
  ///
  /// # Errors
  ///
  /// Returns why the configs cannot be searched or `system.ltx` does not resolve.
  pub fn open_configs(&self) -> XrfResult<GameConfigs> {
    GameConfigs::open(LtxProject::open_lean_at_search_opt(
      self.roots.describe(),
      &self.search()?,
      Some(Self::CONFIGS_DIRECTORY),
      LtxProjectOptions::default().with_dialect(self.get_dialect()),
    )?)
  }

  /// The collision form of the level `name` (`levels\<name>\level.cform`), which the physics world collides with.
  ///
  /// # Errors
  ///
  /// Returns why the form cannot be read: the level has none, or it does not parse.
  pub fn read_collision(&self, name: &str) -> XrfResult<LevelCformGeometry> {
    LevelCformFile::read_geometry_from_bytes(self.search()?.read(&format!(
      "{}\\{name}\\{}",
      LevelSource::LEVELS_DIRECTORY,
      LevelCformFile::FILE_NAME
    ))?)
  }

  /// Its game material library (`gamemtl.xr`), which the level's faces name their materials in.
  ///
  /// # Errors
  ///
  /// Returns why the library cannot be read: the tree has none, or it does not parse.
  pub fn read_materials(&self) -> XrfResult<GameMtlFile> {
    GameMtlFile::read_from_bytes(self.search()?.read(GameMtlFile::FILE_NAME)?)
  }

  /// The names of the levels it holds geometry for, in order.
  ///
  /// # Errors
  ///
  /// Returns why the tree's levels cannot be listed.
  pub fn list_levels(&self) -> XrfResult<Vec<String>> {
    let mut names: Vec<String> = LevelEntry::list(&self.search()?)?
      .into_iter()
      .filter(|entry| entry.has_geometry)
      .map(|entry| entry.name)
      .collect();

    names.sort();

    Ok(names)
  }
}
