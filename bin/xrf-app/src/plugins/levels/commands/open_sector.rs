use std::sync::Arc;

use tauri::State;
use xrf_level::{LevelSector, LevelSectorComposition};
use xrf_spawn::XRayByteOrder;
use xrf_visual::{SectorDescription, SectorPackage, SectorPacker};

use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{LevelState, SelectedLevel, sectors_of};

/// Pack one sector of the open level and report what it became.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_sector"))]
#[tauri::command(rename = "open_sector")]
pub async fn levels_open_sector(
  session_id: SessionId,
  sector_id: SessionId,
  sector: u32,
  state: State<'_, LevelState>,
) -> TauriResult<SessionSnapshot<SectorDescription>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let sectors: &[LevelSector] = sectors_of(&current.level);

  let Some(named) = sectors.get(sector as usize) else {
    return Err(format!(
      "The open level names {} sectors, so sector {sector} is not one of them",
      sectors.len()
    ));
  };

  current.packed.begin_open(sector_id)?;

  log::info!("Packing sector {sector}");

  let composition: LevelSectorComposition = LevelSectorComposition::of(&current.visuals, named.root);

  let package: SectorPackage = {
    let mut geometry = current
      .geometry
      .lock()
      .map_err(|error| format!("Failed to read the level's geometry: {error}"))?;

    SectorPacker::new(&current.visuals, current.level.shaders.as_ref(), &mut geometry)
      .pack::<XRayByteOrder>(sector, &composition)
  };

  log::info!(
    "Packed sector {sector}: {} vertices, {} indices, {} draws, {} bytes",
    package.description.vertex_count,
    package.description.index_count,
    package.description.sections.len(),
    package.buffer.len()
  );

  let opened: Arc<SessionSnapshot<SectorPackage>> = current.packed.commit_open(sector_id, package)?;

  Ok(opened.map(|package| package.description.clone()))
}
