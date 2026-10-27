use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Instant;

use xrf_app_core::session::{SessionPart, SessionParts, SessionSnapshot};
use xrf_ltx::{Ltx, LtxResolution};
use xrf_material::XrayReducedTextures;
use xrf_vfs::{XrayInputRecord, XraySearch};

use crate::level::{LevelConfigs, LevelSource, SelectedLevel};
use crate::parts::{LevelSpawn, LevelSpawnPart};
use crate::report::LevelReport;

/// What a change event calls a level's sections when they go stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_SECTIONS_PART: &str = "sections";

/// The sections of the game's resolved `system.ltx` a level's spawned objects name, which their lights are read from:
/// the game's configs as a level reads them.
pub(crate) struct LevelSectionsPart;

impl SessionPart for LevelSectionsPart {
  type Value = Result<Arc<Ltx>, String>;

  const NAME: &'static str = LEVEL_SECTIONS_PART;
}

impl LevelSectionsPart {
  /// The sections the open level's spawned objects name and [`XrayReducedTextures::SECTION`], kept with the level:
  /// read as it opened, or the first time anything asks where the open did not.
  ///
  /// # Errors
  ///
  /// Returns the reason the spawn or the configs cannot be read, which every later ask answers with again.
  pub fn get(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> Result<Arc<Ltx>, String> {
    current.parts.get_or_record::<Self>(search, |search| {
      let spawn: Arc<LevelSpawn> = LevelSpawnPart::get(current, search)?;

      Self::read(&current.source, search, &LevelConfigs::of(current), &spawn)
    })
  }

  /// The open level's sections where they were read, as the open reads them; asking reads nothing.
  pub fn find(current: &SessionSnapshot<SelectedLevel>) -> Option<Arc<Ltx>> {
    current.parts.get::<Self>()?.ok()
  }

  /// `parts` with the sections an open read from `inputs` kept as the level's.
  pub fn with(parts: SessionParts, sections: Result<Arc<Ltx>, String>, inputs: XrayInputRecord) -> SessionParts {
    parts.with::<Self>(sections, inputs)
  }

  /// The sections a level's spawned objects name and [`XrayReducedTextures::SECTION`], out of `system.ltx` with its
  /// includes merged and every section's parents resolved, read through `search` with the dialect the level is opened
  /// with.
  ///
  /// # Errors
  ///
  /// Returns the reason the configs cannot be read.
  pub fn read(
    source: &LevelSource,
    search: &XraySearch,
    configs: &LevelConfigs,
    spawn: &LevelSpawn,
  ) -> Result<Arc<Ltx>, String> {
    let started: Instant = Instant::now();
    let resolved: Arc<LtxResolution> = configs
      .open(search)?
      .system_ltx()
      .map_err(|error| format!("Failed to read 'system.ltx' for {}: {error}", source.get_label()))?;
    let names: BTreeSet<&str> = spawn
      .objects
      .iter()
      .map(|object| object.section.as_str())
      .chain([XrayReducedTextures::SECTION])
      .collect();
    let mut kept: Ltx = Ltx::new();
    let mut missing: Vec<&str> = Vec::new();

    for name in names {
      match resolved.ltx.section(name) {
        Some(section) => {
          kept.entry(name.to_owned()).or_insert_with(|| section.clone());
        }
        // A game whose configs reduce no texture is not missing anything.
        None if name == XrayReducedTextures::SECTION => {}
        None => missing.push(name),
      }
    }

    LevelReport::sections(source, kept.len(), resolved.ltx.len(), started);
    LevelReport::missing_sections_named(source, &missing);

    Ok(Arc::new(kept))
  }
}
