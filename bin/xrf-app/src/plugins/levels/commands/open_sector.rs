use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::State;
use xrf_level::{LevelSector, LevelSectorComposition};
use xrf_spawn::XRayByteOrder;
use xrf_visual::{SectorAttributes, SectorDescription, SectorPackage, SectorPacker};

use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::report::report_packed_sector;
use crate::plugins::levels::state::{LevelState, PackedSector, SelectedLevel};

/// What the viewer draws a level surface with, which is what a pack is worth carrying.
const DRAWN_ATTRIBUTES: SectorAttributes = SectorAttributes {
  binormals: false,
  colors: false,
  hemi: false,
  lightmap_uvs: true,
  normals: true,
  tangents: false,
  uvs: true,
};

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
  let sectors: &[LevelSector] = current.get_sectors();

  let Some(named) = sectors.get(sector as usize) else {
    return Err(format!(
      "The open level names {} sectors, so sector {sector} is not one of them",
      sectors.len()
    ));
  };

  current.packed.begin_open(sector_id)?;

  let started: Instant = Instant::now();
  let composition: LevelSectorComposition = LevelSectorComposition::of(&current.visuals, named.root);

  log::info!(
    "Packing sector {sector} of root {}: {} drawables, {} hierarchies",
    named.root,
    composition.drawables.len(),
    composition.hierarchies.len()
  );

  current.packed.require_opening(sector_id)?;

  let package: SectorPackage = SectorPacker::new(&current.visuals, current.level.shaders.as_ref(), &current.geometry)
    .pack::<XRayByteOrder>(sector, &composition, DRAWN_ATTRIBUTES);

  report_packed_sector(&package, started);

  let opened: Arc<SessionSnapshot<PackedSector>> = current.packed.commit_open(
    sector_id,
    PackedSector {
      buffer: Mutex::new(Some(package.buffer)),
      description: package.description.clone(),
    },
  )?;

  Ok(opened.map(|packed| packed.description.clone()))
}
