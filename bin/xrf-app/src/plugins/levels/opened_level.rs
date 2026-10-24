use std::sync::Arc;

use xrf_chunk::InMemoryChunkDataSource;
use xrf_engine_target::XrayEngineResolution;
use xrf_level::LevelGeomSource;
use xrf_ltx::{Ltx, LtxDialect};
use xrf_material::{XraySurfaceDescriptor, XrayTextureScope};
use xrf_vfs::{XrayInputRecord, XrayRoots, XraySearch};
use xrf_visual::SectorOutline;

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionDerived, SessionParts, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::parts::{
  LevelGeometryPart, LevelMapPart, LevelSectionsPart, LevelSpawnPart, LevelStartPart, LevelSurfacesPart,
  LevelTexturesPart,
};
use crate::plugins::levels::read_level::ReadLevel;
use crate::plugins::levels::report::report_start;
use crate::plugins::levels::state::{
  LevelMapDescription, LevelSource, LevelSpawn, LevelStart, LevelTextureReference, SelectedLevel,
};

/// A compiled level read through the shared asset state: what an open commits, and what a commit's follow derives
/// again once it moved a file the level was read from. Kept together until the level is built.
pub(in crate::plugins::levels) struct OpenedLevel {
  read: ReadLevel,
  /// The render geometry and what it was read from.
  geometry: (Arc<LevelGeomSource<InMemoryChunkDataSource>>, XrayInputRecord),
  /// How the surfaces are drawn, and what they were resolved from.
  surfaces: (Arc<Vec<XraySurfaceDescriptor>>, XrayInputRecord),
  /// What the surfaces' textures resolved to, and what they were resolved from: the surfaces' inputs among them.
  textures: (Arc<Vec<LevelTextureReference>>, XrayInputRecord),
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
  /// Where the map lies, and what it was read from.
  level_map: (Option<LevelMapDescription>, XrayInputRecord),
}

impl OpenedLevel {
  /// Reads the level `source` names through `search`, recording what the level itself is read from as its inputs and
  /// keeping its geometry, surfaces, textures, map, spawn, the sections it names and where it starts as parts with their
  /// own; `engine` decides the engine its configs are read as, through the same recording search.
  pub fn read(
    search: XraySearch,
    source: LevelSource,
    roots: XrayRoots,
    dialect: Arc<dyn LtxDialect>,
    engine: impl FnOnce(&XraySearch, &XrayRoots) -> XrayEngineResolution,
  ) -> TauriResult<SessionDerived<SelectedLevel>> {
    let inputs: XrayInputRecord = XrayInputRecord::new();
    let search: XraySearch = search.with_recording(&inputs);
    let opened: OpenedLevel = Self::open(&source, &search, (&roots, &dialect))?;
    let engine: XrayEngineResolution = engine(&search, &roots);

    report_start(&source, opened.start.as_ref());

    let mut parts: SessionParts = LevelSpawnPart::with(SessionParts::new(), opened.spawn, opened.spawn_inputs);

    parts = LevelStartPart::with(parts, opened.start, opened.start_inputs);
    parts = LevelGeometryPart::with(parts, opened.geometry.0, opened.geometry.1);
    parts = LevelSurfacesPart::with(parts, opened.surfaces.0, opened.surfaces.1);
    parts = LevelTexturesPart::with(parts, opened.textures.0, opened.textures.1);
    parts = LevelMapPart::with(parts, opened.level_map.0, opened.level_map.1);

    if let Some(sections) = opened.sections {
      parts = LevelSectionsPart::with(parts, sections, opened.sections_inputs);
    }

    Ok(SessionDerived {
      value: SelectedLevel {
        dialect,
        engine,
        level: opened.read.level,
        outlines: opened.outlines,
        roots,
        source,
        visuals: opened.read.visuals,
      },
      inputs,
      parts,
    })
  }

  /// The open level read again over the source, roots and dialect it was opened with, as the engine it was read as.
  pub fn derive(
    current: &SessionSnapshot<SelectedLevel>,
    assets: &AssetMountState,
  ) -> TauriResult<SessionDerived<SelectedLevel>> {
    Self::read(
      assets.search(&current.roots)?,
      current.source.clone(),
      current.roots.clone(),
      Arc::clone(&current.dialect),
      |_, _| current.engine.clone(),
    )
  }

  /// Reads the level, its geometry and its spawn side by side, outlines its sectors and decides where it opens.
  ///
  /// `search` records what the level itself is read from: its bundle. The geometry, the surfaces, the textures, the
  /// map, the spawn, the sections and the start, kept as parts, record their own.
  fn open(source: &LevelSource, search: &XraySearch, configs: (&XrayRoots, &Arc<dyn LtxDialect>)) -> TauriResult<Self> {
    let scope: XrayTextureScope = source.get_texture_scope();
    let geometry_inputs: XrayInputRecord = XrayInputRecord::new();
    let surfaces_inputs: XrayInputRecord = XrayInputRecord::new();
    let textures_inputs: XrayInputRecord = XrayInputRecord::new();
    let map_inputs: XrayInputRecord = XrayInputRecord::new();
    let spawn_inputs: XrayInputRecord = XrayInputRecord::new();
    let sections_inputs: XrayInputRecord = XrayInputRecord::new();
    let start_inputs: XrayInputRecord = XrayInputRecord::new();
    // The spawn, what stands over the places it has an actor at, and the configs' sections it names are read beside the
    // level rather than after it: the renderer's first texture asks whether the configs reduce it.
    let ((read, geometry), (spawn, open, sections)) = rayon::join(
      || {
        (
          ReadLevel::read(source, search),
          LevelGeometryPart::read(source, &search.clone().with_recording(&geometry_inputs)),
        )
      },
      || {
        let spawn: Result<Arc<LevelSpawn>, String> =
          LevelSpawnPart::read(source, &search.clone().with_recording(&spawn_inputs));

        // Where the level starts is decided over its spawn, and the sections are those the spawn names.
        start_inputs.extend_from(&spawn_inputs);
        sections_inputs.extend_from(&spawn_inputs);

        let open: Vec<bool> = spawn.as_deref().map_or_else(
          |_| Vec::new(),
          |spawn| {
            LevelStartPart::find_open(
              source,
              &search.clone().with_recording(&start_inputs),
              &LevelStartPart::list_eyes(spawn),
            )
          },
        );

        let sections: Option<Result<Arc<Ltx>, String>> = spawn.as_deref().ok().map(|spawn| {
          LevelSectionsPart::read(source, &search.clone().with_recording(&sections_inputs), configs, spawn)
        });

        (spawn, open, sections)
      },
    );
    let read: ReadLevel = read?;
    // A level without its geometry draws nothing, so it does not open; one that loses it later keeps its part's failure.
    let geometry: Arc<LevelGeomSource<InMemoryChunkDataSource>> = Arc::new(geometry?);
    let surfaces: Vec<XraySurfaceDescriptor> =
      LevelSurfacesPart::resolve(&read.level, &search.clone().with_recording(&surfaces_inputs), &scope);

    // The textures are resolved over the surfaces, so what moves the surfaces moves them too.
    textures_inputs.extend_from(&surfaces_inputs);

    let textures: Vec<LevelTextureReference> = LevelTexturesPart::resolve(
      &read.level,
      &surfaces,
      &search.clone().with_recording(&textures_inputs),
      &scope,
    );
    let outlines: Vec<SectorOutline> = read
      .level
      .sectors
      .as_ref()
      .map_or(&[][..], |chunk| &chunk.sectors)
      .iter()
      .enumerate()
      .map(|(index, sector)| SectorOutline::of(&read.visuals, index as u32, sector.root))
      .collect();
    let start: Option<LevelStart> = LevelStartPart::resolve(
      spawn.as_deref().ok(),
      SectorOutline::merge_bounds(&outlines).as_ref(),
      &open,
    );

    Ok(Self {
      geometry: (geometry, geometry_inputs),
      level_map: (
        LevelMapPart::read(source, &search.clone().with_recording(&map_inputs)),
        map_inputs,
      ),
      outlines,
      read,
      sections,
      sections_inputs,
      spawn,
      spawn_inputs,
      start,
      start_inputs,
      surfaces: (Arc::new(surfaces), surfaces_inputs),
      textures: (Arc::new(textures), textures_inputs),
    })
  }
}
