use std::sync::{Arc, Mutex};

use tauri::State;
use xrf_vfs::XrayRoots;
use xrf_visual::SectorOutline;

use crate::core::assets::AssetMountState;
use crate::core::session::{Session, SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::read::{ReadLevel, read_source};
use crate::plugins::levels::state::{LevelSource, LevelState, SelectedLevel, SelectedLevelDescription, sectors_of};

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

  log::info!("Opening level: {}", source.label());

  let roots: XrayRoots = roots.centred_on(source.physical_path());
  let read: ReadLevel = assets.with_probe(&roots, |probe| read_source(&source, probe))??;

  let outlines: Vec<SectorOutline> = sectors_of(&read.level)
    .iter()
    .enumerate()
    .map(|(index, sector)| SectorOutline::of(&read.visuals, index as u32, sector.root))
    .collect();

  log::info!(
    "Opened level {} of {} sectors and {} visuals",
    source.label(),
    outlines.len(),
    read.visuals.visuals.len()
  );

  let selected: Arc<SessionSnapshot<SelectedLevel>> = state.selected.commit_open(
    session_id,
    SelectedLevel {
      geometry: Mutex::new(read.geometry),
      level: read.level,
      outlines,
      packed: Session::new("level sector"),
      roots,
      source,
      visuals: read.visuals,
    },
  )?;

  Ok(selected.map(SelectedLevel::describe))
}
