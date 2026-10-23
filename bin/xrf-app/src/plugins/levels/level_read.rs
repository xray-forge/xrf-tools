//! A compiled level read through the shared asset state: what an open commits, and what a commit's follow derives
//! again once it moved a file the level was read from.

use std::sync::Arc;

use xrf_engine_target::XrayEngineResolution;
use xrf_ltx::{Ltx, LtxDialect};
use xrf_material::{XraySurfaceDescriptor, XrayTextureScope};
use xrf_vfs::{XrayInputRecord, XrayRoots, XraySearch};
use xrf_visual::SectorOutline;

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionDerived, SessionParts, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::configs::{read_level_sections, with_level_sections};
use crate::plugins::levels::level_map::read_level_map;
use crate::plugins::levels::read::{ReadLevel, read_source};
use crate::plugins::levels::report::report_start;
use crate::plugins::levels::spawn::{read_source_spawn, with_level_spawn};
use crate::plugins::levels::start::{
  find_open, get_level_start, list_level_start_eyes, resolve_level_start, with_level_start,
};
use crate::plugins::levels::state::{
  LevelMapDescription, LevelSource, LevelSpawn, LevelStart, LevelTextureReference, SelectedLevel,
  SelectedLevelDescription,
};
use crate::plugins::levels::surfaces::resolve_surfaces;
use crate::plugins::levels::textures::resolve_level_textures;

/// What the read reads and decides, kept together until the level is built.
struct OpenedLevel {
  read: ReadLevel,
  textures: Vec<LevelTextureReference>,
  surfaces: Vec<XraySurfaceDescriptor>,
  spawn: Result<Arc<LevelSpawn>, String>,
  /// What the spawn was read from.
  spawn_inputs: XrayInputRecord,
  /// The sections the spawn names and the textures the configs reduce, none where the spawn could not be read.
  sections: Option<Result<Arc<Ltx>, String>>,
  /// What the sections were read from.
  sections_inputs: XrayInputRecord,
  outlines: Vec<SectorOutline>,
  start: Option<LevelStart>,
  /// What the start was decided from: the spawn, and the collision form over its eyes.
  start_inputs: XrayInputRecord,
  level_map: Option<LevelMapDescription>,
}

/// Reads the level `source` names through `search`, recording what the level itself is read from as its inputs and
/// keeping the spawn, the sections it names and where the level starts as parts with their own; `engine` decides the
/// engine its configs are read as, through the same recording search.
pub(in crate::plugins::levels) fn read_level(
  search: XraySearch,
  source: LevelSource,
  roots: XrayRoots,
  dialect: Arc<dyn LtxDialect>,
  engine: impl FnOnce(&XraySearch, &XrayRoots) -> XrayEngineResolution,
) -> TauriResult<SessionDerived<SelectedLevel>> {
  let inputs: XrayInputRecord = XrayInputRecord::new();
  let search: XraySearch = search.with_recording(&inputs);
  let opened: OpenedLevel = open(&source, &search, (&roots, &dialect))?;
  let engine: XrayEngineResolution = engine(&search, &roots);

  report_start(&source, opened.start.as_ref());

  let mut parts: SessionParts = with_level_spawn(SessionParts::new(), opened.spawn, opened.spawn_inputs);

  parts = with_level_start(parts, opened.start, opened.start_inputs);

  if let Some(sections) = opened.sections {
    parts = with_level_sections(parts, sections, opened.sections_inputs);
  }

  Ok(SessionDerived {
    value: SelectedLevel {
      dialect,
      engine,
      level_map: opened.level_map,
      geometry: opened.read.geometry,
      level: opened.read.level,
      outlines: opened.outlines,
      roots,
      source,
      surfaces: opened.surfaces,
      textures: opened.textures,
      visuals: opened.read.visuals,
    },
    inputs,
    parts,
  })
}

/// The open level read again over the source, roots and dialect it was opened with, as the engine it was read as.
pub(in crate::plugins::levels) fn derive_level(
  current: &SessionSnapshot<SelectedLevel>,
  assets: &AssetMountState,
) -> TauriResult<SessionDerived<SelectedLevel>> {
  read_level(
    assets.search(&current.roots)?,
    current.source.clone(),
    current.roots.clone(),
    Arc::clone(&current.dialect),
    |_, _| current.engine.clone(),
  )
}

/// What the viewer is shown of the open level under its snapshot's identity, where it starts decided again through
/// `search` where a commit let the start go.
pub(in crate::plugins::levels) fn describe_level(
  current: &SessionSnapshot<SelectedLevel>,
  search: &XraySearch,
) -> SessionSnapshot<SelectedLevelDescription> {
  current.answer(current.describe(get_level_start(current, search)))
}

/// Reads the level and its spawn side by side, outlines its sectors and decides where it opens.
///
/// `search` records what the level itself is read from; the spawn, the sections and the start, kept as parts, record
/// their own.
fn open(
  source: &LevelSource,
  search: &XraySearch,
  configs: (&XrayRoots, &Arc<dyn LtxDialect>),
) -> TauriResult<OpenedLevel> {
  let scope: XrayTextureScope = source.get_texture_scope();
  let spawn_inputs: XrayInputRecord = XrayInputRecord::new();
  let sections_inputs: XrayInputRecord = XrayInputRecord::new();
  let start_inputs: XrayInputRecord = XrayInputRecord::new();
  // The spawn, what stands over the places it has an actor at, and the configs' sections it names are read beside the
  // level rather than after it: the renderer's first texture asks whether the configs reduce it.
  let (read, (spawn, open, sections)) = rayon::join(
    || read_source(source, search),
    || {
      let spawn: Result<Arc<LevelSpawn>, String> =
        read_source_spawn(source, &search.clone().with_recording(&spawn_inputs));

      // Where the level starts is decided over its spawn, and the sections are those the spawn names.
      start_inputs.extend_from(&spawn_inputs);
      sections_inputs.extend_from(&spawn_inputs);

      let open: Vec<bool> = spawn.as_deref().map_or_else(
        |_| Vec::new(),
        |spawn| {
          find_open(
            source,
            &search.clone().with_recording(&start_inputs),
            &list_level_start_eyes(spawn),
          )
        },
      );

      let sections: Option<Result<Arc<Ltx>, String>> = spawn
        .as_deref()
        .ok()
        .map(|spawn| read_level_sections(source, &search.clone().with_recording(&sections_inputs), configs, spawn));

      (spawn, open, sections)
    },
  );
  let read: ReadLevel = read?;
  let surfaces: Vec<XraySurfaceDescriptor> = resolve_surfaces(&read.level, search, &scope);
  let textures: Vec<LevelTextureReference> = resolve_level_textures(&read.level, &surfaces, search, &scope);
  let outlines: Vec<SectorOutline> = read
    .level
    .sectors
    .as_ref()
    .map_or(&[][..], |chunk| &chunk.sectors)
    .iter()
    .enumerate()
    .map(|(index, sector)| SectorOutline::of(&read.visuals, index as u32, sector.root))
    .collect();
  let start: Option<LevelStart> = resolve_level_start(
    spawn.as_deref().ok(),
    SectorOutline::merge_bounds(&outlines).as_ref(),
    &open,
  );

  Ok(OpenedLevel {
    level_map: read_level_map(source, search),
    outlines,
    read,
    sections,
    sections_inputs,
    spawn,
    spawn_inputs,
    start,
    start_inputs,
    surfaces,
    textures,
  })
}
