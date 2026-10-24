//! Reading a game's environment configs for a level's weather, suns, thunderbolts and ambients.

use std::sync::Arc;

use xrf_environment::{EnvironmentCatalog, EnvironmentReadOptions, EnvironmentReader};
use xrf_ltx::{LtxDialect, LtxProject, LtxProjectOptions};
use xrf_vfs::{XrayRoots, XraySearch};

/// Where the game keeps its configs.
pub const CONFIGS_DIRECTORY: &str = "configs";

/// Opens the configs `search` reaches as a project that reads only what is asked of it, through the search, so they are
/// recorded where it records; `roots` names it in reports.
///
/// # Errors
///
/// Returns the reason the configs directory cannot be named.
pub fn open_configs(
  roots: &XrayRoots,
  search: &XraySearch,
  dialect: Arc<dyn LtxDialect>,
) -> Result<LtxProject, String> {
  LtxProject::open_lean_at_search_opt(
    roots.describe(),
    search,
    Some(CONFIGS_DIRECTORY),
    LtxProjectOptions::default().with_dialect(dialect),
  )
  .map_err(|error| format!("Failed to open the game's configs: {error}"))
}

/// Reads every environment config of a project as one engine does.
///
/// # Errors
///
/// Returns the reason the configs cannot be listed.
pub fn read_catalog(project: &LtxProject, options: &EnvironmentReadOptions) -> Result<EnvironmentCatalog, String> {
  EnvironmentReader::read_opt(project, options)
    .map_err(|error| format!("Failed to read the game's environment configs: {error}"))
}
