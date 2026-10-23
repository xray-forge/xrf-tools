//! The game's configs as a level reads them: `system.ltx`, resolved, for the sections its spawned objects name.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Instant;

use xrf_ltx::{Ltx, LtxDialect, LtxResolution};
use xrf_material::XrayReducedTextures;
use xrf_vfs::{XrayInputRecord, XrayRoots, XraySearch};

use crate::core::session::{SessionPart, SessionParts, SessionSnapshot};
use crate::plugins::levels::catalog::open_configs;
use crate::plugins::levels::report::report_sections;
use crate::plugins::levels::spawn::get_level_spawn;
use crate::plugins::levels::state::{LevelSource, LevelSpawn, SelectedLevel};

/// The sections of the game's resolved `system.ltx` a level's spawned objects name, which their lights are read from.
struct LevelSectionsPart;

impl SessionPart for LevelSectionsPart {
  type Value = Result<Arc<Ltx>, String>;

  const NAME: &'static str = "sections";
}

/// The sections the open level's spawned objects name and [`XrayReducedTextures::SECTION`], kept with the level: read
/// as it opened, or the first time anything asks where the open did not.
///
/// # Errors
///
/// Returns the reason the spawn or the configs cannot be read, which every later ask answers with again.
pub fn get_level_sections(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Result<Arc<Ltx>, String> {
  current.parts.get_or_record::<LevelSectionsPart>(search, |search| {
    let spawn: Arc<LevelSpawn> = get_level_spawn(current, search)?;

    read_level_sections(&current.source, search, (&current.roots, &current.dialect), &spawn)
  })
}

/// The open level's sections where they were read, as the open reads them; asking reads nothing.
pub fn find_level_sections(current: &SessionSnapshot<SelectedLevel>) -> Option<Arc<Ltx>> {
  current.parts.get::<LevelSectionsPart>()?.ok()
}

/// `parts` with the sections an open read from `inputs` kept as the level's.
pub fn with_level_sections(
  parts: SessionParts,
  sections: Result<Arc<Ltx>, String>,
  inputs: XrayInputRecord,
) -> SessionParts {
  parts.with::<LevelSectionsPart>(sections, inputs)
}

/// The sections a level's spawned objects name and [`XrayReducedTextures::SECTION`], out of `system.ltx` with its
/// includes merged and every section's parents resolved, read through `search` with the dialect the level is opened
/// with.
///
/// # Errors
///
/// Returns the reason the configs cannot be read.
pub fn read_level_sections(
  source: &LevelSource,
  search: &XraySearch,
  (roots, dialect): (&XrayRoots, &Arc<dyn LtxDialect>),
  spawn: &LevelSpawn,
) -> Result<Arc<Ltx>, String> {
  let started: Instant = Instant::now();
  let resolved: Arc<LtxResolution> = open_configs(roots, search, dialect.clone())?
    .system_ltx()
    .map_err(|error| format!("Failed to read 'system.ltx' for {}: {error}", source.get_label()))?;
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
