//! The game's configs as a level reads them: `system.ltx`, resolved, for the sections its spawned objects name.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Instant;

use xrf_dltx::select_ltx_dialect;
use xrf_ltx::{Ltx, LtxProject, LtxProjectOptions, LtxResolution};

use crate::plugins::levels::report::report_sections;
use crate::plugins::levels::state::{LevelSpawn, SelectedLevel};

/// Where the game keeps its configs, which `system.ltx` is read from.
const CONFIGS_DIRECTORY: &str = "configs";

/// The sections the open level's spawned objects name, out of `system.ltx` with its includes merged and every
/// section's parents resolved, read the first time anything asks and kept with the level.
///
/// The rest of the resolution is let go once read: a resolved tree is tens of megabytes, and the level reads a handful
/// of keys of the sections its own objects name. Read with the DLTX dialect, which is what Anomaly's engine resolves
/// with and which resolves a vanilla tree as the vanilla engine does.
///
/// # Errors
///
/// Returns the reason the configs cannot be read, which every later ask answers with again.
pub fn get_level_sections(current: &SelectedLevel, spawn: &LevelSpawn) -> Result<Arc<Ltx>, String> {
  current
    .sections
    .get_or_init(|| {
      let started: Instant = Instant::now();
      let resolved: Arc<LtxResolution> = read_system_ltx(current)?;
      let names: BTreeSet<&str> = spawn.objects.iter().map(|object| object.section.as_str()).collect();
      let mut kept: Ltx = Ltx::new();

      for name in names {
        if let Some(section) = resolved.ltx.section(name) {
          kept.entry(name.to_owned()).or_insert_with(|| section.clone());
        }
      }

      report_sections(&current.source, kept.len(), resolved.ltx.len(), started);

      Ok(Arc::new(kept))
    })
    .clone()
}

fn read_system_ltx(current: &SelectedLevel) -> Result<Arc<LtxResolution>, String> {
  let project: LtxProject = LtxProject::open_lean_at_roots_opt(
    &current.roots,
    Some(CONFIGS_DIRECTORY),
    LtxProjectOptions {
      dialect: select_ltx_dialect(true),
      ..Default::default()
    },
  )
  .map_err(|error| format!("Failed to mount the configs of {}: {error}", current.source.get_label()))?;

  project.system_ltx().map_err(|error| {
    format!(
      "Failed to read 'system.ltx' for {}: {error}",
      current.source.get_label()
    )
  })
}
