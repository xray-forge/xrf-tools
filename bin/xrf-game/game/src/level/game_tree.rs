use std::path::Path;
use std::sync::Arc;

use xrf_chunk::{ChunkReader, InMemoryChunkDataSource};
use xrf_dltx::DltxDialect;
use xrf_engine_target::{XrayEngine, XrayEngineChoice, XrayEngineDetector, XrayEngineResolution};
use xrf_error::XrfResult;
use xrf_game_alife::AlifeSettings;
use xrf_level_source::LevelEntry;
use xrf_ltx::{Ltx, LtxDialect};
use xrf_spawn::{SpawnFile, SpawnPlacedObjects};
use xrf_vfs::{XrayAssetType, XrayCachePolicy, XrayMountMode, XrayRoots, XraySearch, XrayVfs, XrayVfsHost};

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

  /// The configs file holding `[alife]`.
  const ALIFE_FILE: &'static str = "configs\\alife.ltx";

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

  /// Every object its spawn (`spawns\all.spawn`) places, each with its level, those no reader takes skipped: what a
  /// new game starts from.
  ///
  /// # Errors
  ///
  /// Returns why the spawn cannot be read: the tree has none, or its chunks or graph do not read.
  pub fn read_spawn(&self) -> XrfResult<SpawnPlacedObjects> {
    let bytes: Vec<u8> = self.search()?.read(Self::SPAWN_FILE)?;
    let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(bytes)?.read_children()?;

    SpawnFile::read_objects_from_chunks(&chunks)
  }

  /// The configs' `[alife]` (`configs\alife.ltx`), Call of Pripyat's where the tree holds none or it does not read.
  pub fn read_alife_settings(&self) -> AlifeSettings {
    let text: XrfResult<String> = self
      .search()
      .and_then(|search| search.read(Self::ALIFE_FILE))
      .map(|bytes| String::from_utf8_lossy(&bytes).into_owned());

    match text.and_then(|text| Ltx::read_from_str(&text)) {
      Ok(ltx) => AlifeSettings::read_ltx(&ltx),
      Err(error) => {
        log::warn!(
          "{} does not read ({error}); A-Life runs on Call of Pripyat's",
          Self::ALIFE_FILE
        );

        AlifeSettings::default()
      }
    }
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
