use std::collections::HashMap;
use std::sync::Arc;

use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelSector, LevelSectorComposition};
use xrf_material::XraySurfaceDescriptor;
use xrf_renderer::{RenderAssetSource, RenderLevelSource};
use xrf_visual::{SectorPackage, SectorPacker};

use crate::core::assets::{AssetMountState, read_located_asset};
use crate::core::session::SessionSnapshot;
use crate::plugins::levels::drawn_attributes::DRAWN_ATTRIBUTES;
use crate::plugins::levels::state::SelectedLevel;

/// The open level as the native renderer draws it: sectors packed when its loader asks, textures read from the roots
/// the level was opened in.
pub struct LevelRenderSource {
  level: Arc<SessionSnapshot<SelectedLevel>>,
  assets: AssetMountState,
  /// Where each texture reference the level's surfaces bind resolved to at open, `None` for one resolving to nothing.
  textures: HashMap<String, Option<String>>,
}

impl LevelRenderSource {
  pub fn new(level: Arc<SessionSnapshot<SelectedLevel>>, assets: AssetMountState) -> Self {
    let textures: HashMap<String, Option<String>> = level
      .textures
      .iter()
      .map(|texture| (texture.reference.clone(), texture.logical_path.clone()))
      .collect();

    Self {
      level,
      assets,
      textures,
    }
  }
}

impl RenderAssetSource for LevelRenderSource {
  fn read_texture(&self, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    let Some(Some(logical_path)) = self.textures.get(reference) else {
      return Ok(None);
    };

    self
      .assets
      .with_probe(&self.level.roots, |probe| read_located_asset(probe, logical_path))
      .map_err(XrfError::new_asset_error)?
      .map(Some)
  }
}

impl RenderLevelSource for LevelRenderSource {
  fn get_sector_count(&self) -> u32 {
    self.level.get_sectors().len() as u32
  }

  fn pack_sector(&self, sector: u32) -> XrfResult<SectorPackage> {
    let sectors: &[LevelSector] = self.level.get_sectors();
    let root: u32 = sectors
      .get(sector as usize)
      .ok_or_else(|| XrfError::new_not_found_error(format!("The level has no sector {sector}")))?
      .root;
    let composition: LevelSectorComposition = LevelSectorComposition::of(&self.level.visuals, root);

    Ok(
      SectorPacker::new(
        &self.level.visuals,
        self.level.level.shaders.as_ref(),
        &self.level.geometry,
      )
      .pack::<XRayByteOrder>(sector, &composition, DRAWN_ATTRIBUTES),
    )
  }

  fn get_surfaces(&self) -> &[XraySurfaceDescriptor] {
    &self.level.surfaces
  }
}
