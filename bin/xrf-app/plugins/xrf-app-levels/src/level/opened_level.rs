use std::sync::Arc;

use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionDerived, SessionParts, SessionSnapshot};
use xrf_chunk::InMemoryChunkDataSource;
use xrf_engine_target::XrayEngineResolution;
use xrf_level::LevelGeomSource;
use xrf_ltx::{Ltx, LtxDialect};
use xrf_material::{XraySurfaceDescriptor, XrayTextureScope};
use xrf_vfs::{XrayInputRecord, XrayRoots, XraySearch};
use xrf_visual::SectorOutline;

use crate::level::{LevelBundle, LevelConfigs, LevelSource, SelectedLevel};
use crate::parts::{
  LevelGeometryPart, LevelMapDescription, LevelMapPart, LevelSectionsPart, LevelSpawn, LevelSpawnPart, LevelStart,
  LevelStartPart, LevelSurfacesPart, LevelTextureReference, LevelTexturesPart,
};
use crate::report::LevelReport;

/// How a compiled level is read through the shared asset state: what an open commits, and what a commit's follow
/// derives again once it moved a file the level was read from.
pub(crate) struct OpenedLevel;

impl OpenedLevel {
  /// Reads the level `source` names through `search`, recording what the level itself is read from as its inputs and
  /// keeping its geometry, surfaces, textures, map, spawn, the sections it names and where it starts as parts with their
  /// own; `engine` decides the engine its configs are read as, through the same recording search.
  pub(crate) fn read(
    search: XraySearch,
    source: LevelSource,
    roots: XrayRoots,
    dialect: Arc<dyn LtxDialect>,
    engine: impl FnOnce(&XraySearch, &XrayRoots) -> XrayEngineResolution,
  ) -> TauriResult<SessionDerived<SelectedLevel>> {
    profiling::scope!("level open");

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

  /// The open level read again over the source, roots and dialect it was opened with, as the engine it was read as.
  pub(crate) fn derive(
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

  /// Reads the level, its geometry and its spawn side by side, outlines its sectors, decides where it opens, and keeps
  /// each read as its part as it returns.
  ///
  /// `search` records what the level itself is read from: its bundle. The geometry, the surfaces, the textures, the
  /// map, the spawn, the sections and the start, kept as parts, record their own.
  fn open(
    source: &LevelSource,
    search: &XraySearch,
    configs: &LevelConfigs,
  ) -> TauriResult<(LevelBundle, Vec<SectorOutline>, SessionParts)> {
    let scope: XrayTextureScope = source.get_texture_scope();
    let geometry_inputs: XrayInputRecord = XrayInputRecord::new();
    let surfaces_inputs: XrayInputRecord = XrayInputRecord::new();
    let textures_inputs: XrayInputRecord = XrayInputRecord::new();
    let map_inputs: XrayInputRecord = XrayInputRecord::new();
    let spawn_inputs: XrayInputRecord = XrayInputRecord::new();
    let sections_inputs: XrayInputRecord = XrayInputRecord::new();
    let start_inputs: XrayInputRecord = XrayInputRecord::new();
    // Two lines of reads, each waiting only on what it needs: the level, its geometry and then the surfaces and textures
    // its shader table names; and beside them the spawn, then what stands over the places it has an actor at and the
    // configs' sections it names, side by side. The renderer's first texture asks whether the configs reduce it, so the
    // sections are read with the level rather than after it.
    let ((dressed, geometry), (spawn, open, sections)) = rayon::join(
      || {
        let (bundle, geometry) = rayon::join(
          || {
            profiling::scope!("level bundle");

            LevelBundle::read(source, search)
          },
          || {
            profiling::scope!("level geometry");

            LevelGeometryPart::read(source, &search.clone().with_recording(&geometry_inputs))
          },
        );
        let dressed: TauriResult<(LevelBundle, Vec<XraySurfaceDescriptor>, Vec<LevelTextureReference>)> =
          bundle.map(|bundle| {
            profiling::scope!("level surfaces and textures");

            let surfaces: Vec<XraySurfaceDescriptor> =
              LevelSurfacesPart::resolve(&bundle.level, &search.clone().with_recording(&surfaces_inputs), &scope);

            // The textures are resolved over the surfaces, so what moves the surfaces moves them too.
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
        let spawn: TauriResult<Arc<LevelSpawn>> = {
          profiling::scope!("level spawn");

          LevelSpawnPart::read(source, &search.clone().with_recording(&spawn_inputs))
        };

        // Where the level starts is decided over its spawn, and the sections are those the spawn names.
        start_inputs.extend_from(&spawn_inputs);
        sections_inputs.extend_from(&spawn_inputs);

        let Ok(found) = spawn.as_deref() else {
          return (spawn, Vec::new(), None);
        };
        let (open, sections): (Vec<bool>, TauriResult<Arc<Ltx>>) = rayon::join(
          || {
            profiling::scope!("level start");

            LevelStartPart::find_open(source, &search.clone().with_recording(&start_inputs), found)
          },
          || {
            profiling::scope!("level sections");

            LevelSectionsPart::read(source, &search.clone().with_recording(&sections_inputs), configs, found)
          },
        );

        (spawn, open, Some(sections))
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
      profiling::scope!("level map");

      LevelMapPart::read(source, &search.clone().with_recording(&map_inputs))
    };

    parts = LevelMapPart::with(parts, map, map_inputs);

    if let Some(sections) = sections {
      parts = LevelSectionsPart::with(parts, sections, sections_inputs);
    }

    Ok((bundle, outlines, parts))
  }
}
