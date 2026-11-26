use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_chunk::InMemoryChunkDataSource;
use xrf_engine_target::XrayEngineResolution;
use xrf_error::XrfResult;
use xrf_job::parallel;
use xrf_level::LevelGeomSource;
use xrf_ltx::{Ltx, LtxDialect, LtxResolution};
use xrf_material::{XraySurfaceDescriptor, XrayTextureScope};
use xrf_session::{SessionDerived, SessionParts, SessionSnapshot};
use xrf_vfs::{XrayInputRecord, XrayRoots, XraySearch, XrayVfsHost};
use xrf_visual::SectorOutline;

use crate::level::{LevelBundle, LevelConfigs, LevelSource, SelectedLevel};
use crate::parts::{
  LevelGeometryPart, LevelMapDescription, LevelMapPart, LevelSectionsPart, LevelSpawn, LevelSpawnPart, LevelStart,
  LevelStartPart, LevelSurfacesPart, LevelTextureReference, LevelTexturesPart,
};
use crate::report::LevelReport;

/// How a compiled level is read through the shared asset state: what an open commits, and what a commit's follow
/// derives again once it moved a file the level was read from.
pub struct OpenedLevel;

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
  ) -> XrfResult<SessionDerived<SelectedLevel>> {
    xrf_profiler::scope!("level open");

    let inputs: XrayInputRecord = XrayInputRecord::new();
    let search: XraySearch = search.with_recording(&inputs);
    let configs: LevelConfigs = LevelConfigs {
      roots: roots.clone(),
      dialect: Arc::clone(&dialect),
    };
    let (bundle, outlines, parts): (LevelBundle, Vec<SectorOutline>, SessionParts) =
      Self::open(&source, &search, &configs)?;
    let engine: XrayEngineResolution = engine(&search, &roots);

    Ok(SessionDerived {
      value: SelectedLevel {
        dialect,
        engine,
        level: bundle.level,
        outlines,
        roots,
        source,
        visuals: bundle.visuals,
      },
      inputs,
      parts,
    })
  }

  /// The open level read again over the source, roots and dialect it was opened with, as the engine it was read as,
  /// through `host`, which mounts its roots where they are not yet.
  pub fn derive(
    current: &SessionSnapshot<SelectedLevel>,
    host: &XrayVfsHost,
  ) -> XrfResult<SessionDerived<SelectedLevel>> {
    Self::read(
      host.search(&current.roots)?,
      current.source.clone(),
      current.roots.clone(),
      Arc::clone(&current.dialect),
      |_, _| current.engine.clone(),
    )
  }

  /// Reads the level, its geometry and its spawn side by side, outlines its sectors, decides where it opens, and keeps
  /// each read as its part as it returns.
  ///
  /// `search` records what the level itself is read from: its bundle. The geometry, the surfaces, the textures, the
  /// map, the spawn, the sections and the start, kept as parts, record their own.
  fn open(
    source: &LevelSource,
    search: &XraySearch,
    configs: &LevelConfigs,
  ) -> XrfResult<(LevelBundle, Vec<SectorOutline>, SessionParts)> {
    let scope: XrayTextureScope = source.get_texture_scope();
    let geometry_inputs: XrayInputRecord = XrayInputRecord::new();
    let surfaces_inputs: XrayInputRecord = XrayInputRecord::new();
    let textures_inputs: XrayInputRecord = XrayInputRecord::new();
    let map_inputs: XrayInputRecord = XrayInputRecord::new();
    let spawn_inputs: XrayInputRecord = XrayInputRecord::new();
    let sections_inputs: XrayInputRecord = XrayInputRecord::new();
    let start_inputs: XrayInputRecord = XrayInputRecord::new();
    // Two parallel lines of reads: the level and its geometry, then its surfaces and textures; and the spawn, then its
    // start places, beside the game's configs. The configs are the longest read, so they start at once rather than
    // after the spawn; the renderer's first texture needs them, so they are read with the level, not after it.
    let ((dressed, geometry), (spawn, open, sections)) = parallel::join(
      || {
        let (bundle, geometry) = parallel::join(
          || {
            xrf_profiler::scope!("level bundle");

            LevelBundle::read(source, search)
          },
          || {
            xrf_profiler::scope!("level geometry");

            LevelGeometryPart::read(source, &search.clone().with_recording(&geometry_inputs))
          },
        );
        let dressed: XrfResult<(LevelBundle, Vec<XraySurfaceDescriptor>, Vec<LevelTextureReference>)> =
          bundle.map(|bundle| {
            xrf_profiler::scope!("level surfaces and textures");

            let surfaces: Vec<XraySurfaceDescriptor> =
              LevelSurfacesPart::resolve(&bundle.level, &search.clone().with_recording(&surfaces_inputs), &scope);

            // Textures resolve over the surfaces, so the surfaces' inputs invalidate them too.
            textures_inputs.extend_from(&surfaces_inputs);

            let textures: Vec<LevelTextureReference> = LevelTexturesPart::resolve(
              &bundle.level,
              &surfaces,
              &search.clone().with_recording(&textures_inputs),
              &scope,
            );

            (bundle, surfaces, textures)
          });

        (dressed, geometry)
      },
      || {
        let (spawned, (resolved, read_in)) = parallel::join(
          || -> (XrfResult<Arc<LevelSpawn>>, Vec<bool>) {
            let spawn: XrfResult<Arc<LevelSpawn>> = {
              xrf_profiler::scope!("level spawn");

              LevelSpawnPart::read(source, &search.clone().with_recording(&spawn_inputs), configs)
            };

            // The start is found over the spawn, so its inputs invalidate it too.
            start_inputs.extend_from(&spawn_inputs);

            let open: Vec<bool> = spawn.as_deref().map_or_else(
              |_| Vec::new(),
              |found| {
                xrf_profiler::scope!("level start");

                LevelStartPart::find_open(source, &search.clone().with_recording(&start_inputs), found)
              },
            );

            (spawn, open)
          },
          || -> (XrfResult<Arc<LtxResolution>>, Duration) {
            xrf_profiler::scope!("level configs");

            let started: Instant = Instant::now();
            let resolved: XrfResult<Arc<LtxResolution>> =
              LevelSectionsPart::resolve(source, &search.clone().with_recording(&sections_inputs), configs);

            (resolved, started.elapsed())
          },
        );
        let (spawn, open) = spawned;

        // The sections kept are those the spawn names.
        sections_inputs.extend_from(&spawn_inputs);

        let sections: Option<XrfResult<Arc<Ltx>>> = spawn
          .as_deref()
          .ok()
          .map(|found| resolved.map(|resolved| LevelSectionsPart::keep(source, &resolved, read_in, found)));

        (spawn, open, sections)
      },
    );
    let (bundle, surfaces, textures): (LevelBundle, Vec<XraySurfaceDescriptor>, Vec<LevelTextureReference>) = dressed?;
    // A level without its geometry draws nothing, so it does not open; one that loses it later keeps its part's failure.
    let geometry: Arc<LevelGeomSource<InMemoryChunkDataSource>> = Arc::new(geometry?);
    let outlines: Vec<SectorOutline> = bundle
      .level
      .get_sectors()
      .iter()
      .enumerate()
      .map(|(index, sector)| SectorOutline::of(&bundle.visuals, index as u32, sector.root))
      .collect();
    let start: Option<LevelStart> = LevelStartPart::resolve(
      spawn.as_deref().ok(),
      SectorOutline::merge_bounds(&outlines).as_ref(),
      &open,
    );

    LevelReport::start(source, start.as_ref());

    let mut parts: SessionParts = LevelSpawnPart::with(SessionParts::new(), spawn, spawn_inputs);

    parts = LevelStartPart::with(parts, start, start_inputs);
    parts = LevelGeometryPart::with(parts, geometry, geometry_inputs);
    parts = LevelSurfacesPart::with(parts, Arc::new(surfaces), surfaces_inputs);
    parts = LevelTexturesPart::with(parts, Arc::new(textures), textures_inputs);
    let map: Option<LevelMapDescription> = {
      xrf_profiler::scope!("level map");

      LevelMapPart::read(source, &search.clone().with_recording(&map_inputs))
    };

    parts = LevelMapPart::with(parts, map, map_inputs);

    if let Some(sections) = sections {
      parts = LevelSectionsPart::with(parts, sections, sections_inputs);
    }

    Ok((bundle, outlines, parts))
  }
}
