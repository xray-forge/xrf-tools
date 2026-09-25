use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use tauri::State;
use xrf_material::XraySurfaceDescriptor;
use xrf_vfs::XrayRoots;
use xrf_visual::SectorOutline;

use crate::core::assets::AssetMountState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::read::{ReadLevel, read_source};
use crate::plugins::levels::report::report_open;
use crate::plugins::levels::state::{
  LevelSource, LevelState, LevelTextureReference, PackedDetails, PackedSectors, SelectedLevel, SelectedLevelDescription,
};
use crate::plugins::levels::surfaces::resolve_surfaces;
use crate::plugins::levels::textures::resolve_textures;

/// Select a compiled level and report what it is built out of, without reading any of its geometry.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_level"))]
#[tauri::command(rename = "open_level")]
pub async fn levels_open_level(
  session_id: SessionId,
  source: LevelSource,
  roots: XrayRoots,
  state: State<'_, LevelState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<SessionSnapshot<SelectedLevelDescription>> {
  state.selected.begin_open(session_id)?;

  let started: Instant = Instant::now();

  log::info!("Opening level: {}", source.get_label());

  let roots: XrayRoots = roots.centred_on(source.get_physical_path());
  let directory: Option<String> = source.get_logical_directory();
  let (read, textures, surfaces) = assets.with_probe(&roots, |probe| {
    let read: ReadLevel = read_source(&source, probe)?;
    let surfaces: Vec<XraySurfaceDescriptor> = resolve_surfaces(&read.level, probe);
    let textures: Vec<LevelTextureReference> = resolve_textures(&read.level, &surfaces, probe, directory.as_deref());

    TauriResult::Ok((read, textures, surfaces))
  })??;

  let outlines: Vec<SectorOutline> = read
    .level
    .sectors
    .as_ref()
    .map_or(&[][..], |chunk| &chunk.sectors)
    .iter()
    .enumerate()
    .map(|(index, sector)| SectorOutline::of(&read.visuals, index as u32, sector.root))
    .collect();

  let selected: Arc<SessionSnapshot<SelectedLevel>> = state.selected.commit_open(
    session_id,
    SelectedLevel {
      details: PackedDetails::new(),
      spawn: OnceLock::new(),
      configs: OnceLock::new(),
      spawn_visuals: Mutex::new(HashMap::new()),
      geometry: read.geometry,
      level: read.level,
      outlines,
      packed: PackedSectors::new(),
      roots,
      source,
      surfaces,
      textures,
      visuals: read.visuals,
    },
  )?;

  report_open(&selected.value, started);

  Ok(selected.map(SelectedLevel::describe))
}
