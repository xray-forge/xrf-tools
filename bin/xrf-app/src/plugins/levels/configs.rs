//! The game's configs as a level reads them: `system.ltx`, resolved, for the sections its spawned objects name.

use std::sync::Arc;
use std::time::Instant;

use xrf_dltx::select_ltx_dialect;
use xrf_ltx::{LtxProject, LtxProjectOptions, LtxResolution};

use crate::plugins::levels::state::SelectedLevel;

/// Where the game keeps its configs, which `system.ltx` is read from.
const CONFIGS_DIRECTORY: &str = "configs";

/// The open level's `system.ltx`, with its includes merged and every section's parents resolved, read the first time
/// anything asks and kept with the level.
///
/// Read with the DLTX dialect, which is what Anomaly's engine resolves with and which resolves a vanilla tree as the
/// vanilla engine does.
///
/// # Errors
///
/// Returns the reason the configs cannot be read, which every later ask answers with again.
pub fn get_level_configs(current: &SelectedLevel) -> Result<Arc<LtxResolution>, String> {
  current
    .configs
    .get_or_init(|| {
      let started: Instant = Instant::now();
      let project: LtxProject = LtxProject::open_lean_at_roots(
        &current.roots,
        Some(CONFIGS_DIRECTORY),
        LtxProjectOptions {
          dialect: select_ltx_dialect(true),
          ..Default::default()
        },
      )
      .map_err(|error| format!("Failed to mount the configs of {}: {error}", current.source.get_label()))?;
      let resolved: Arc<LtxResolution> = project.system_ltx().map_err(|error| {
        format!(
          "Failed to read 'system.ltx' for {}: {error}",
          current.source.get_label()
        )
      })?;

      log::info!(
        "Read the configs of {}: {} sections in {:?}",
        current.source.get_label(),
        resolved.ltx.sections().count(),
        started.elapsed()
      );

      Ok(resolved)
    })
    .clone()
}
