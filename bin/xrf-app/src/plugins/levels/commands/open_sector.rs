use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::State;
use xrf_chunk::XRayByteOrder;
use xrf_level::{LevelSector, LevelSectorComposition};
use xrf_visual::{SectorDescription, SectorPackage, SectorPacker};

use crate::core::execution::ExecutionState;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::drawn_attributes::DRAWN_ATTRIBUTES;
use crate::plugins::levels::report::{report_packed_sector, report_packing_sector};
use crate::plugins::levels::state::{LevelState, PackedSector, SelectedLevel};

/// Pack one sector of the open level and report what it became.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_sector"))]
#[tauri::command(rename = "open_sector")]
pub async fn levels_open_sector(
  session_id: SessionId,
  sector_id: SessionId,
  sector: u32,
  state: State<'_, LevelState>,
  execution: State<'_, ExecutionState>,
) -> TauriResult<SessionSnapshot<SectorDescription>> {
  let current: Arc<SessionSnapshot<SelectedLevel>> = state.selected.require(session_id)?;
  let sectors: &[LevelSector] = current.get_sectors();

  let Some(root) = sectors.get(sector as usize).map(|named| named.root) else {
    return Err(format!(
      "The open level names {} sectors, so sector {sector} is not one of them",
      sectors.len()
    ));
  };

  let packing: Arc<SessionSnapshot<SelectedLevel>> = Arc::clone(&current);
  let package: SectorPackage = execution
    .run_blocking("Packing the level sector", move || {
      let started: Instant = Instant::now();
      let composition: LevelSectorComposition = LevelSectorComposition::of(&packing.visuals, root);

      report_packing_sector(sector, root, &composition);

      let package: SectorPackage = SectorPacker::new(
        &packing.visuals,
        packing.level.shaders.as_ref(),
        &packing.geometry,
      )
      .pack::<XRayByteOrder>(sector, &composition, DRAWN_ATTRIBUTES);

      report_packed_sector(&package, started);

      package
    })
    .await?;

  current.packed.park(
    sector_id,
    PackedSector {
      buffer: Mutex::new(Some(package.buffer)),
      sector,
    },
  )?;

  Ok(SessionSnapshot {
    session_id: sector_id,
    value: package.description,
  })
}
