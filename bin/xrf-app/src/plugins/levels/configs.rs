//! The game's configs as a level reads them: `system.ltx`, resolved, for the sections its spawned objects name.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Instant;

use xrf_ltx::{Ltx, LtxDialect, LtxProject, LtxProjectOptions, LtxResolution};
use xrf_material::XrayReducedTextures;
use xrf_vfs::XrayRoots;

use crate::plugins::levels::report::report_sections;
use crate::plugins::levels::state::{LevelSource, LevelSpawn, SelectedLevel};

/// Where the game keeps its configs, which `system.ltx` is read from.
const CONFIGS_DIRECTORY: &str = "configs";

/// The sections the open level's spawned objects name and [`XrayReducedTextures::SECTION`], kept with the level: read
/// as it opened, or the first time anything asks where the open did not.
///
/// # Errors
///
/// Returns the reason the configs cannot be read, which every later ask answers with again.
pub fn get_level_sections(current: &SelectedLevel, spawn: &LevelSpawn) -> Result<Arc<Ltx>, String> {
  current
    .sections
    .get_or_init(|| read_level_sections(&current.source, (&current.roots, &current.dialect), spawn))
    .clone()
}

/// The sections a level's spawned objects name and [`XrayReducedTextures::SECTION`], out of `system.ltx` with its
/// includes merged and every section's parents resolved, read with the dialect the level is opened with.
///
/// The rest of the resolution is let go once read: a resolved tree is tens of megabytes, and the level reads a handful
/// of keys of the sections its own objects name.
///
/// # Errors
///
/// Returns the reason the configs cannot be read.
pub fn read_level_sections(
  source: &LevelSource,
  (roots, dialect): (&XrayRoots, &Arc<dyn LtxDialect>),
  spawn: &LevelSpawn,
) -> Result<Arc<Ltx>, String> {
  let started: Instant = Instant::now();
  let resolved: Arc<LtxResolution> = read_system_ltx(source, roots, dialect)?;
  let names: BTreeSet<&str> = spawn
    .objects
    .iter()
    .map(|object| object.section.as_str())
    .chain([XrayReducedTextures::SECTION])
    .collect();
  let mut kept: Ltx = Ltx::new();

  for name in names {
    if let Some(section) = resolved.ltx.section(name) {
      kept.entry(name.to_owned()).or_insert_with(|| section.clone());
    }
  }

  report_sections(source, kept.len(), resolved.ltx.len(), started);

  Ok(Arc::new(kept))
}

fn read_system_ltx(
  source: &LevelSource,
  roots: &XrayRoots,
  dialect: &Arc<dyn LtxDialect>,
) -> Result<Arc<LtxResolution>, String> {
  let project: LtxProject = LtxProject::open_lean_at_roots_opt(
    roots,
    Some(CONFIGS_DIRECTORY),
    LtxProjectOptions::default().with_dialect(dialect.clone()),
  )
  .map_err(|error| format!("Failed to mount the configs of {}: {error}", source.get_label()))?;

  project
    .system_ltx()
    .map_err(|error| format!("Failed to read 'system.ltx' for {}: {error}", source.get_label()))
}
