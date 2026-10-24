use serde::Serialize;
use xrf_level::{LevelFile, LevelSector};

/// What the level is built out of beside its geometry: the sectors it culls by and the lights it loads.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelStructureReport {
  sectors: usize,
  portals: usize,
  /// Portals named by a sector, which is twice the portals when every one joins two of them.
  sector_portal_references: usize,
  lights: usize,
  /// Whether the compiler wrote the directional light the loader keeps as the sun.
  has_sun: bool,
}

impl LevelStructureReport {
  pub fn of(level: &LevelFile) -> Self {
    let sectors: &[LevelSector] = level.sectors.as_ref().map_or(&[], |chunk| &chunk.sectors);

    Self {
      has_sun: level.lights.as_ref().is_some_and(|chunk| chunk.get_sun().is_some()),
      lights: level.lights.as_ref().map_or(0, |chunk| chunk.lights.len()),
      portals: level.portals.as_ref().map_or(0, |chunk| chunk.portals.len()),
      sector_portal_references: sectors.iter().map(|sector| sector.portals.len()).sum(),
      sectors: sectors.len(),
    }
  }
}
