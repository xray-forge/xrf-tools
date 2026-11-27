use std::path::Path;
use std::sync::Arc;

use xrf_dltx::DltxDialect;
use xrf_engine_target::{XrayEngine, XrayEngineChoice, XrayEngineDetector, XrayEngineResolution};
use xrf_error::XrfResult;
use xrf_level_source::LevelEntry;
use xrf_ltx::LtxDialect;
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
