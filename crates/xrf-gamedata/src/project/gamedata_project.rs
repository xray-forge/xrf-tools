use std::io;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use xrf_chunk::{ChunkReader, InMemoryChunkDataSource, XRayByteOrder};
use xrf_engine_target::{XrayEngineDetector, XrayEngineResolution};
use xrf_error::{XrfError, XrfResult};
use xrf_gamemtl::GameMtlFile;
use xrf_ltx::{LtxProject, LtxProjectOptions};
use xrf_utils::format_path;
use xrf_vfs::{
  XrayAsset, XrayAssetType, XrayCachePolicy, XrayCacheStats, XrayMountMode, XrayPathCollision, XrayReadTraceSummary,
  XrayRoots, XraySearch, XraySearchPlan, XraySkippedMount, XrayVfs,
};

use crate::project::gamedata_project_read_options::GamedataProjectReadOptions;
use crate::project::levels::level_engine_constants::SPAWNS_DIRECTORY;

/// Logical directory holding a project's configs.
pub(crate) const CONFIGS_DIRECTORY: &str = "configs";

/// Root config the engine loads, and the one asset whose absence means this is not a usable project.
const SYSTEM_LTX_LOGICAL_PATH: &str = "configs\\system.ltx";

/// What the one step a project searches is called in a report.
const SEARCH_STEP: &str = "gamedata";

#[derive(Debug)]
pub struct GamedataProject {
  /// The config tree, read through the same snapshot as every check, so they share what it retains.
  pub(crate) ltx_project: LtxProject,
  /// The search every check reads through: the whole tree, as the caller left it once ignored prefixes are out.
  pub(crate) search: XraySearch,
  /// Location shown in output, which for an installation is the game directory rather than any one mount.
  pub(crate) root: PathBuf,
  /// The engine the tree is meant for, which the environment check reads its configs as, and what decided it.
  pub(crate) engine: XrayEngineResolution,
}

impl GamedataProject {
  pub fn root(&self) -> &Path {
    &self.root
  }

  /// The engine the project's configs are read as, and what decided it.
  pub fn get_engine(&self) -> &XrayEngineResolution {
    &self.engine
  }

  /// Opens a project at a path, reading it the way `mode` says.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is not a directory, when the mode cannot plan it, or when the unfiltered project
  /// holds no `configs\system.ltx`.
  pub fn open_with_mode(mode: XrayMountMode, options: &GamedataProjectReadOptions) -> XrfResult<Self> {
    if !Self::is_valid_gamedata_dir(&options.root) {
      return Err(
        io::Error::new(
          ErrorKind::NotFound,
          format!(
            "Invalid gamedata root provided: {}, an existing directory is required",
            format_path(&options.root)
          ),
        )
        .into(),
      );
    }

    let plan: XraySearchPlan = XraySearchPlan::new().with_root_mode(SEARCH_STEP, &options.root, mode)?;

    // Project identity is independent of the inspection filter: a mesh-only check must not be aborted because a caller
    // omitted configs. Resolve through the unfiltered plan so archive-only installations remain valid too.
    let identity: XraySearch = plan.open()?;

    if identity.find(SYSTEM_LTX_LOGICAL_PATH)?.is_none() {
      return Err(
        io::Error::new(
          ErrorKind::NotFound,
          format!(
            "Invalid gamedata provided: {}, nothing resolves '{SYSTEM_LTX_LOGICAL_PATH}'",
            format_path(&options.root)
          ),
        )
        .into(),
      );
    }

    let engine: XrayEngineResolution = options.engine.resolve(|| {
      XrayEngineDetector::detect(&XrayRoots::one(options.root.clone(), mode), |logical_path| {
        identity.read(logical_path).ok()
      })
    });

    // Checks enumerate and read only what the caller left in scope.
    // Retaining motions is what keeps a sweep from re-reading a shared animation bank once per referencing visual.
    let vfs: XrayVfs = XrayVfs::new().with_cache_policy(XrayCachePolicy::verification());

    // Applied after the identity probe, so the account describes the sweep rather than the two reads opening it took.
    let vfs: XrayVfs = if options.is_tracing_reads {
      vfs.with_read_trace()
    } else {
      vfs
    };

    let search: XraySearch = plan.ignoring(&options.ignored)?.open_in(vfs)?;

    let ltx_project: LtxProject = LtxProject::open_at_search_opt(
      // The configs directory, not the game root: this project *is* the config tree, and callers join onto its root.
      options.root.join(CONFIGS_DIRECTORY),
      &search,
      Some(CONFIGS_DIRECTORY),
      LtxProjectOptions {
        dialect: options.dialect.clone(),
        // Several verifiers read `system.ltx` and the same config roots, so resolving one twice would be paid work.
        is_caching_resolutions: true,
        is_with_schemes_check: true,
        is_strict_check: false,
      },
    )
    .map_err(|error| XrfError::new_asset_error(format!("Failed to open gamedata project ltx configs: {}", error)))?;

    Ok(Self {
      engine,
      ltx_project,
      root: options.root.clone(),
      search,
    })
  }

  /// Opens a project, treating the path as an installation only when it declares one.
  pub fn open(options: &GamedataProjectReadOptions) -> XrfResult<Self> {
    Self::open_with_mode(XrayMountMode::Auto, options)
  }

  /// The search every check of this project reads through.
  pub(crate) fn search(&self) -> &XraySearch {
    &self.search
  }

  /// How well retention has served this project's reads so far.
  pub fn get_cache_stats(&self) -> XrayCacheStats {
    self.search.get_vfs().get_cache_stats()
  }

  /// What this project physically read, or `None` unless it was opened with `is_tracing_reads`.
  pub fn get_read_trace_summary(&self, hottest: usize) -> Option<XrayReadTraceSummary> {
    self
      .search
      .get_vfs()
      .get_read_trace()
      .map(|trace| trace.get_summary(hottest))
  }

  /// Reads an asset's bytes through the VFS, whether it is loose or inside an archive volume.
  ///
  /// # Errors
  ///
  /// Returns an error when nothing in the project holds the path, or the source cannot read it.
  pub(crate) fn read_bytes(&self, logical_path: &str) -> XrfResult<Vec<u8>> {
    self.search.read(logical_path)
  }

  /// Reads and parses a chunked asset, serving what this project is already holding.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when nothing in the project holds the path, and otherwise whatever reading the asset or
  /// parsing it answers with. A failure is not retained, so each caller of a broken asset reports the real error
  /// rather than a copy of the first one.
  pub(crate) fn read_parsed<T, F>(&self, logical_path: &str, parse: F) -> XrfResult<Arc<T>>
  where
    T: Send + Sync + 'static,
    F: FnOnce(&mut ChunkReader<InMemoryChunkDataSource>) -> XrfResult<T>,
  {
    let asset: XrayAsset = self
      .search
      .find(logical_path)?
      .ok_or_else(|| XrfError::new_not_found_error(format!("no asset '{logical_path}' in the project")))?;

    self
      .search
      .read_parsed(&asset, |bytes| parse(&mut ChunkReader::from_vec(bytes)?))
  }

  /// Files any mount holds but cannot reach, because another file in the same mount claims their engine identity.
  pub fn collisions(&self) -> Vec<XrayPathCollision> {
    self.search.list_collisions()
  }

  /// Sources this project's installation declared that could not be opened.
  pub fn skipped_mounts(&self) -> &[XraySkippedMount] {
    self.search.get_vfs().get_skipped_mounts()
  }
}

/// Asset access bound to this project's search.
impl GamedataProject {
  /// The winning asset for a logical path, or `None` when nothing in the project holds it.
  ///
  /// # Errors
  ///
  /// Returns an error when the path is not a valid X-Ray logical path.
  pub(crate) fn find(&self, logical_path: &str) -> XrfResult<Option<XrayAsset>> {
    self.search.find(logical_path)
  }

  /// Size of an asset without reading it, for a gate that exists to avoid parsing a truncated file.
  pub(crate) fn size(&self, logical_path: &str) -> Option<u64> {
    self.search.read_size(logical_path)
  }

  /// Every asset the project resolves, one per logical path, ordered by that path.
  pub(crate) fn entries(&self) -> Vec<XrayAsset> {
    self.search.list_files()
  }

  /// Every asset whose extension identifies one kind, wherever in the tree it lives.
  pub(crate) fn entries_of_type(&self, asset_type: XrayAssetType) -> Vec<XrayAsset> {
    self.search.list_assets(asset_type)
  }

  /// Every asset whose logical path ends with `suffix` on a component boundary.
  ///
  /// # Errors
  ///
  /// Returns an error when `suffix` is not a valid X-Ray logical path fragment.
  pub(crate) fn entries_with_suffix(&self, suffix: &str) -> XrfResult<Vec<XrayAsset>> {
    let mut found: Vec<XrayAsset> = Vec::new();

    for asset in self.search.list_files() {
      if asset.get_logical_path().ends_with(suffix)? {
        found.push(asset);
      }
    }

    Ok(found)
  }

  /// Resolves a raw engine reference of one kind under that kind's directory and extension.
  ///
  /// # Errors
  ///
  /// Returns an error when the kind has no canonical home, or the reference is not a valid X-Ray path.
  pub(crate) fn resolve(&self, asset_type: XrayAssetType, reference: &str) -> XrfResult<Option<XrayAsset>> {
    Ok(self.search.resolve(asset_type, reference)?.into_asset())
  }

  /// Resolves every asset of one kind a reference names, which may be a `*` mask.
  ///
  /// # Errors
  ///
  /// Returns an error when the kind has no canonical home, the reference is invalid, or a mask carries more than one `*`.
  pub(crate) fn resolve_all(&self, asset_type: XrayAssetType, reference: &str) -> XrfResult<Vec<XrayAsset>> {
    Ok(self.search.resolve(asset_type, reference)?.into_assets())
  }

  /// Logical paths of the spawn files under `spawns`, the merged sets that carry the game graph and alife objects.
  pub(crate) fn spawn_paths(&self) -> Vec<String> {
    self
      .entries_of_type(XrayAssetType::Spawn)
      .into_iter()
      .filter(|location| location.get_logical_path().is_under(SPAWNS_DIRECTORY).unwrap_or(false))
      .map(|location| location.get_logical_path().to_string())
      .collect()
  }

  /// The game material library, which several checks resolve material names and numbers against.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when the project holds no library, and otherwise whatever reading it answers with.
  pub(crate) fn game_materials(&self) -> XrfResult<Arc<GameMtlFile>> {
    self.read_parsed(GameMtlFile::FILE_NAME, GameMtlFile::read_from_chunk::<XRayByteOrder, _>)
  }

  /// Resolves an OGF reference under the `meshes` namespace.
  ///
  /// # Errors
  ///
  /// Returns an error when the reference is not a valid X-Ray path.
  pub(crate) fn ogf(&self, reference: &str) -> XrfResult<Option<XrayAsset>> {
    self.resolve(XrayAssetType::Ogf, reference)
  }

  /// Resolves an OMF reference under the `meshes` namespace.
  ///
  /// # Errors
  ///
  /// Returns an error when the reference is not a valid X-Ray path.
  pub(crate) fn omf(&self, reference: &str) -> XrfResult<Option<XrayAsset>> {
    self.resolve(XrayAssetType::Omf, reference)
  }

  /// Resolves a texture reference, appending `.dds` or replacing its authoring extension.
  ///
  /// # Errors
  ///
  /// Returns an error when the reference is not a valid X-Ray path.
  pub(crate) fn dds_texture(&self, reference: &str) -> XrfResult<Option<XrayAsset>> {
    self.resolve(XrayAssetType::Dds, reference)
  }

  /// Reads an asset the project already resolved.
  ///
  /// # Errors
  ///
  /// Returns an error when no mount holds the asset's container, or the source cannot read it.
  pub(crate) fn read_resolved(&self, asset: &XrayAsset) -> XrfResult<Vec<u8>> {
    self.search.read_asset(asset)
  }

  /// Opens a chunk reader over an asset the project already resolved.
  ///
  /// # Errors
  ///
  /// Returns an error when the asset cannot be read or holds no chunk.
  pub(crate) fn read_resolved_chunks(&self, asset: &XrayAsset) -> XrfResult<ChunkReader<InMemoryChunkDataSource>> {
    ChunkReader::from_vec(self.read_resolved(asset)?)
  }
}

impl GamedataProject {
  pub fn is_valid_gamedata_dir<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists() && path.as_ref().is_dir()
  }
}
