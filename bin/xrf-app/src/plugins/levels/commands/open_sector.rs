use std::sync::{Arc, Mutex};
use std::time::Instant;

use tauri::State;
use xrf_level::{LevelSector, LevelSectorComposition};
use xrf_spawn::XRayByteOrder;
use xrf_visual::{SectorDescription, SectorInstanceGroup, SectorPackage, SectorPacker};

use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::levels::state::{LevelState, PackedSector, SelectedLevel};

/// Bytes past which a packed sector is worth saying something about: it is a level whose sectors are not a streaming
/// unit, and a viewer holding several of them is in trouble before it runs out of memory.
const LARGE_SECTOR_BYTES: usize = 128 * 1024 * 1024;

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

  let package: SectorPackage = {
    let mut geometry = current
      .geometry
      .lock()
      .map_err(|error| format!("Failed to read the level's geometry: {error}"))?;

    SectorPacker::new(&current.visuals, current.level.shaders.as_ref(), &mut geometry)
      .pack::<XRayByteOrder>(sector, &composition)
  };

  report(&package, started);

  let opened: Arc<SessionSnapshot<PackedSector>> = current.packed.commit_open(
    sector_id,
    PackedSector {
      buffer: Mutex::new(Some(package.buffer)),
      description: package.description.clone(),
    },
  )?;

  Ok(opened.map(|packed| packed.description.clone()))
}

/// Says what one sector came to, and says it louder when it came to too much.
fn report(package: &SectorPackage, started: Instant) {
  let description: &SectorDescription = &package.description;
  let instances: u32 = description
    .instances
    .iter()
    .map(|group: &SectorInstanceGroup| group.instance_count)
    .sum();

  log::info!(
    "Packed sector {} in {}: {} vertices, {} indices, {} draws, {} instanced meshes standing {} times, {}",
    description.sector,
    xrf_utils::format_duration(started.elapsed()),
    description.geometry.vertex_count,
    description.geometry.index_count,
    description.sections.len(),
    description.instances.len(),
    instances,
    xrf_utils::format_bytes(package.buffer.len() as u64)
  );

  if !description.skipped.is_empty() {
    log::warn!(
      "Sector {} left out {} drawables, first: {}",
      description.sector,
      description.skipped.len(),
      description.skipped[0].reason
    );
  }

  if package.buffer.len() >= LARGE_SECTOR_BYTES {
    log::warn!(
      "Sector {} packed to {}, which is past what a viewer should hold several of: this level's sectors are not a streaming unit",
      description.sector,
      xrf_utils::format_bytes(package.buffer.len() as u64)
    );
  }
}
