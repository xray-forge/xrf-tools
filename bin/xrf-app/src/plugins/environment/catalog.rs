//! Reading a game's environment configs, which the environment and levels domains both do.

use std::sync::Arc;

use xrf_engine_target::XrayEngine;
use xrf_environment::{EnvironmentCatalog, EnvironmentReadOptions, EnvironmentReader};
use xrf_ltx::{LtxDialect, LtxProject, LtxProjectOptions};
use xrf_vfs::XrayRoots;

/// Where the game keeps its configs.
pub const CONFIGS_DIRECTORY: &str = "configs";

/// Opens the configs of some roots as a project that reads only what is asked of it.
///
/// # Errors
///
/// Returns the reason the roots cannot be mounted.
pub fn open_configs(roots: &XrayRoots, dialect: Arc<dyn LtxDialect>) -> Result<LtxProject, String> {
  LtxProject::open_lean_at_roots_opt(
    roots,
    Some(CONFIGS_DIRECTORY),
    LtxProjectOptions::default().with_dialect(dialect),
  )
  .map_err(|error| format!("Failed to mount the game's configs: {error}"))
}

/// Reads every environment config of a project as one engine does.
///
/// # Errors
///
/// Returns the reason the configs cannot be listed.
pub fn read_catalog(
  project: &LtxProject,
  engine: XrayEngine,
  options: &EnvironmentReadOptions,
) -> Result<EnvironmentCatalog, String> {
  EnvironmentReader::read_opt(project, engine, options)
    .map_err(|error| format!("Failed to read the game's environment configs: {error}"))
}
