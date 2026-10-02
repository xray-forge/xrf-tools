use xrf_error::XrfResult;
use xrf_material::XraySurfaceDescriptor;
use xrf_visual::SectorPackage;

use crate::host::render_asset_source::RenderAssetSource;

/// An open level as the renderer draws it: the application packs its sectors on demand, on the renderer's loader
/// threads, and reads the files they name.
pub trait RenderLevelSource: RenderAssetSource {
  /// How many sectors the level has.
  fn get_sector_count(&self) -> u32;

  /// One sector's geometry, packed, called from loader threads side by side.
  ///
  /// # Errors
  ///
  /// Returns an error when the sector cannot be packed.
  fn pack_sector(&self, sector: u32) -> XrfResult<SectorPackage>;

  /// How each entry of the level's shader table is drawn, in its order: what a sector's surface names by id.
  fn get_surfaces(&self) -> &[XraySurfaceDescriptor];
}
