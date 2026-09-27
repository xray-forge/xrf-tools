use std::sync::{Arc, OnceLock};
use std::time::Instant;

use tauri::State;
use xrf_chunk::XRayByteOrder;
use xrf_level::LevelCformFile;
use xrf_material::XraySurfaceDescriptor;
use xrf_math::Vector3d;
use xrf_vfs::{XrayLogicalPath, XrayProbe, XrayRoots};
use xrf_visual::SectorOutline;

use crate::core::assets::AssetMountState;
use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::read::{ReadLevel, read_optional_file, read_source};
use crate::plugins::levels::report::{report_open, report_opening, report_start};
use crate::plugins::levels::spawn::read_source_spawn;
use crate::plugins::levels::start::{list_level_start_eyes, resolve_level_start};
use crate::plugins::levels::state::{
  COLLISION_FILE, LevelSource, LevelSpawn, LevelSpawnVisuals, LevelStart, LevelState, LevelTextureReference,
  PackedDetails, PackedSectors, SelectedLevel, SelectedLevelDescription,
};
use crate::plugins::levels::surfaces::resolve_surfaces;
use crate::plugins::levels::textures::resolve_textures;

/// What the open reads and decides off the command's thread, kept together until the level is committed.
struct OpenedLevel {
  read: ReadLevel,
  textures: Vec<LevelTextureReference>,
  surfaces: Vec<XraySurfaceDescriptor>,
  spawn: Result<Arc<LevelSpawn>, String>,
  outlines: Vec<SectorOutline>,
  start: Option<LevelStart>,
}

/// Select a compiled level and report what it is built out of, without reading any of its geometry.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_level"))]
#[tauri::command(rename = "open_level")]
pub async fn levels_open_level(
  session_id: SessionId,
  source: LevelSource,
  roots: XrayRoots,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<SelectedLevelDescription>> {
  state.selected.begin_open(session_id)?;

  let started: Instant = Instant::now();

  report_opening(&source);

  let roots: XrayRoots = roots.centred_on(source.get_physical_path());
  let assets: AssetMountState = AssetMountState::clone(&assets);
  let (opened, source, roots) = execution
    .run_blocking("Opening the level", move || {
      let opened: OpenedLevel = assets.with_probe(&roots, |probe| open(&source, probe))??;

      TauriResult::Ok((opened, source, roots))
    })
    .await??;

  report_start(&source, opened.start.as_ref());

  let selected: Arc<SessionSnapshot<SelectedLevel>> = state.selected.commit_open(
    session_id,
    SelectedLevel {
      details: PackedDetails::new(),
      spawn: opened.spawn.into(),
      sections: OnceLock::new(),
      spawn_visuals: LevelSpawnVisuals::new(),
      geometry: opened.read.geometry,
      level: opened.read.level,
      outlines: opened.outlines,
      packed: PackedSectors::new(),
      roots,
      source,
      start: opened.start,
      surfaces: opened.surfaces,
      textures: opened.textures,
      visuals: opened.read.visuals,
    },
  )?;

  report_open(&selected.value, started);

  Ok(selected.map(SelectedLevel::describe))
}

/// Reads the level and its spawn side by side, outlines its sectors and decides where it opens.
fn open(source: &LevelSource, probe: &XrayProbe) -> TauriResult<OpenedLevel> {
  let directory: Option<XrayLogicalPath> = source.get_logical_directory();
  // The spawn, and what stands over the places it has an actor at, are read beside the level rather than after it.
  let (read, (spawn, open)) = rayon::join(
    || read_source(source, probe),
    || {
      let spawn: Result<Arc<LevelSpawn>, String> = read_source_spawn(source, probe);
      let open: Vec<bool> = spawn.as_deref().map_or_else(
        |_| Vec::new(),
        |spawn| find_open(source, probe, &list_level_start_eyes(spawn)),
      );

      (spawn, open)
    },
  );
  let read: ReadLevel = read?;
  let surfaces: Vec<XraySurfaceDescriptor> = resolve_surfaces(&read.level, probe);
  let textures: Vec<LevelTextureReference> = resolve_textures(&read.level, &surfaces, probe, directory.as_ref());
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
    outlines,
    read,
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
