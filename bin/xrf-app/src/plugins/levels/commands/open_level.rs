use std::sync::Arc;
use std::time::Instant;

use tauri::State;
use xrf_chunk::XRayByteOrder;
use xrf_dltx::select_ltx_dialect;
use xrf_engine_target::XrayEngineResolution;
use xrf_level::LevelCformFile;
use xrf_ltx::{Ltx, LtxDialect};
use xrf_material::{XraySurfaceDescriptor, XrayTextureScope};
use xrf_math::Vector3d;
use xrf_vfs::{XrayProbe, XrayRoots};
use xrf_visual::SectorOutline;

use crate::core::assets::{AssetMountState, detect_probe_engine};
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionParts, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::configs::{read_level_sections, with_level_sections};
use crate::plugins::levels::level_map::read_level_map;
use crate::plugins::levels::read::{ReadLevel, read_optional_file, read_source};
use crate::plugins::levels::report::{report_open, report_opening, report_start};
use crate::plugins::levels::request::LevelOpenRequest;
use crate::plugins::levels::spawn::{read_source_spawn, with_level_spawn};
use crate::plugins::levels::start::{list_level_start_eyes, resolve_level_start};
use crate::plugins::levels::state::{
  COLLISION_FILE, LevelMapDescription, LevelSource, LevelSpawn, LevelStart, LevelState, LevelTextureReference,
  SelectedLevel, SelectedLevelDescription,
};
use crate::plugins::levels::surfaces::resolve_surfaces;
use crate::plugins::levels::textures::resolve_level_textures;

/// What the open reads and decides off the command's thread, kept together until the level is committed.
struct OpenedLevel {
  read: ReadLevel,
  textures: Vec<LevelTextureReference>,
  surfaces: Vec<XraySurfaceDescriptor>,
  spawn: Result<Arc<LevelSpawn>, String>,
  /// The sections the spawn names and the textures the configs reduce, none where the spawn could not be read.
  sections: Option<Result<Arc<Ltx>, String>>,
  outlines: Vec<SectorOutline>,
  start: Option<LevelStart>,
  level_map: Option<LevelMapDescription>,
}

/// Select a compiled level and report what it is built out of, without reading any of its geometry.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_level"))]
#[tauri::command(rename = "open_level")]
pub async fn levels_open_level(
  session_id: SessionId,
  request: LevelOpenRequest,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<SelectedLevelDescription>> {
  let LevelOpenRequest {
    source,
    roots,
    is_dltx,
    engine,
  } = request;

  state.selected.begin_open(session_id)?;

  let started: Instant = Instant::now();

  report_opening(&source);

  let roots: XrayRoots = roots.centred_on(source.get_physical_path());
  let dialect: Arc<dyn LtxDialect> = select_ltx_dialect(is_dltx);
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let (opened, engine, source, roots, dialect) = execution
    .run_blocking("Opening the level", move || {
      let (opened, engine): (OpenedLevel, XrayEngineResolution) = assets.with_fresh_probe(&roots, |probe| {
        open(&source, probe, (&roots, &dialect))
          .map(|opened| (opened, engine.resolve(|| detect_probe_engine(probe, &roots))))
      })??;

      TauriResult::Ok((opened, engine, source, roots, dialect))
    })
    .await??;

  log::info!("Reading level '{}' as {}", source.get_label(), engine.describe());

  report_start(&source, opened.start.as_ref());

  // The spawn and the sections it names are read as the level opens, so they are kept as its parts already.
  let mut parts: SessionParts = with_level_spawn(SessionParts::new(), opened.spawn);

  if let Some(sections) = opened.sections {
    parts = with_level_sections(parts, sections);
  }

  let selected: Arc<SessionSnapshot<SelectedLevel>> = state.selected.commit_open_with(
    session_id,
    SelectedLevel {
      dialect,
      engine,
      level_map: opened.level_map,
      geometry: opened.read.geometry,
      level: opened.read.level,
      outlines: opened.outlines,
      roots,
      source,
      start: opened.start,
      surfaces: opened.surfaces,
      textures: opened.textures,
      visuals: opened.read.visuals,
    },
    parts,
  )?;

  report_open(&selected.value, started);

  Ok(selected.map(SelectedLevel::describe))
}

/// Reads the level and its spawn side by side, outlines its sectors and decides where it opens.
fn open(
  source: &LevelSource,
  probe: &XrayProbe,
  configs: (&XrayRoots, &Arc<dyn LtxDialect>),
) -> TauriResult<OpenedLevel> {
  let scope: XrayTextureScope = source.get_texture_scope();
  // The spawn, what stands over the places it has an actor at, and the configs' sections it names are read beside the
  // level rather than after it: the renderer's first texture asks whether the configs reduce it.
  let (read, (spawn, open, sections)) = rayon::join(
    || read_source(source, probe),
    || {
      let spawn: Result<Arc<LevelSpawn>, String> = read_source_spawn(source, probe);
      let open: Vec<bool> = spawn.as_deref().map_or_else(
        |_| Vec::new(),
        |spawn| find_open(source, probe, &list_level_start_eyes(spawn)),
      );

      let sections: Option<Result<Arc<Ltx>, String>> = spawn
        .as_deref()
        .ok()
        .map(|spawn| read_level_sections(source, configs, spawn));

      (spawn, open, sections)
    },
  );
  let read: ReadLevel = read?;
  let surfaces: Vec<XraySurfaceDescriptor> = resolve_surfaces(&read.level, probe, &scope);
  let textures: Vec<LevelTextureReference> = resolve_level_textures(&read.level, &surfaces, probe, &scope);
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
    level_map: read_level_map(source, probe),
    outlines,
    read,
    sections,
    spawn,
    start,
    surfaces,
    textures,
  })
}

/// Whether nothing of the collision form stands over each eye; every one for a level without a readable form, which
/// covers nothing.
fn find_open(source: &LevelSource, probe: &XrayProbe, eyes: &[Vector3d]) -> Vec<bool> {
  read_optional_file(source, probe, COLLISION_FILE)
    .map_err(|error| log::warn!("{error}"))
    .ok()
    .flatten()
    .and_then(|bytes| {
      LevelCformFile::read_open_above_from_bytes::<XRayByteOrder>(bytes, eyes)
        .map_err(|error| {
          log::warn!(
            "Failed to read '{COLLISION_FILE}' of level '{}': {error}",
            source.get_label()
          )
        })
        .ok()
    })
    .unwrap_or_else(|| vec![true; eyes.len()])
}
