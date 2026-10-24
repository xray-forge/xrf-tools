use std::sync::Arc;

use xrf_ltx::{LtxDialect, LtxProject, LtxProjectOptions};
use xrf_vfs::{XrayRoots, XraySearch};

use crate::level::SelectedLevel;

/// How a level reads the game's configs: over the roots it was opened in, with the dialect it was opened with. Its
/// sections and its environment are both read through it.
#[derive(Clone)]
pub struct LevelConfigs {
  pub roots: XrayRoots,
  pub dialect: Arc<dyn LtxDialect>,
}

impl LevelConfigs {
  /// Where the game keeps its configs.
  pub const DIRECTORY: &'static str = "configs";

  /// The configs `level` reads.
  pub fn of(level: &SelectedLevel) -> Self {
    Self {
      roots: level.roots.clone(),
      dialect: Arc::clone(&level.dialect),
    }
  }

  /// Opens the configs `search` reaches as a project that reads only what is asked of it, through the search, so they
  /// are recorded where it records.
  ///
  /// # Errors
  ///
  /// Returns the reason the configs directory cannot be named.
  pub fn open(&self, search: &XraySearch) -> Result<LtxProject, String> {
    LtxProject::open_lean_at_search_opt(
      self.roots.describe(),
      search,
      Some(Self::DIRECTORY),
      LtxProjectOptions::default().with_dialect(Arc::clone(&self.dialect)),
    )
    .map_err(|error| format!("Failed to open the game's configs: {error}"))
  }
}
