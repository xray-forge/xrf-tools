use xrf_material::XraySurfaceDescriptor;

use crate::host::render_asset_source::RenderAssetSource;

/// An open level as the renderer draws it: what a scene asks of its level while it draws, besides the files it reads.
/// What the scene takes in (sectors, lights, spawn, grass, particles, weather) is read and handed over by its host.
pub trait RenderLevelSource: RenderAssetSource {
  /// How many sectors the level has.
  fn get_sector_count(&self) -> u32;

  /// How each entry of the level's shader table is drawn, in its order: what a sector's surface names by id.
  fn get_surfaces(&self) -> &[XraySurfaceDescriptor];
}
