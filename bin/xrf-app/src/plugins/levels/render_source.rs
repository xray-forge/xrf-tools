use std::collections::HashMap;
use std::sync::{Arc, PoisonError, RwLock};

use xrf_chunk::XRayByteOrder;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelSector, LevelSectorComposition};
use xrf_ltx::Ltx;
use xrf_material::XraySurfaceDescriptor;
use xrf_renderer::{RenderAssetSource, RenderLevelSource};
use xrf_visual::{LightsDescription, SectorPackage, SectorPacker};

use crate::core::assets::{AssetMountState, read_located_asset};
use crate::core::session::SessionSnapshot;
use crate::plugins::levels::configs::get_level_sections;
use crate::plugins::levels::drawn_attributes::DRAWN_ATTRIBUTES;
use crate::plugins::levels::lights::{PackedLevelLights, pack_lights};
use crate::plugins::levels::report::report_missing_sections;
use crate::plugins::levels::spawn::get_level_spawn;
use crate::plugins::levels::state::SelectedLevel;

/// The open level as the native renderer draws it: sectors packed when its loader asks, textures read from the roots
/// the level was opened in.
pub struct LevelRenderSource {
  level: Arc<SessionSnapshot<SelectedLevel>>,
  assets: AssetMountState,
  /// Where each texture reference the level's surfaces bind resolved to at open, and each projector its lights name
  /// once they are read, `None` for one resolving to nothing.
  textures: RwLock<HashMap<String, Option<String>>>,
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
      textures: RwLock::new(textures),
    }
  }
}

impl RenderAssetSource for LevelRenderSource {
  fn read_texture(&self, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    let Some(Some(logical_path)) = self
      .textures
      .read()
      .unwrap_or_else(PoisonError::into_inner)
      .get(reference)
      .cloned()
    else {
      return Ok(None);
    };

    self
      .assets
      .with_probe(&self.level.roots, |probe| read_located_asset(probe, &logical_path))
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

  fn read_lights(&self) -> XrfResult<LightsDescription> {
    let level: &SelectedLevel = &self.level;
    // The sections are read between two probes, as the configs mount a tree of their own.
    let sections: Option<Arc<Ltx>> = self
      .assets
      .with_probe(&level.roots, |probe| get_level_spawn(level, probe))
      .map_err(XrfError::new_asset_error)?
      .and_then(|spawn| get_level_sections(level, &spawn))
      .inspect_err(report_missing_sections)
      .ok();
    let packed: PackedLevelLights = self
      .assets
      .with_probe(&level.roots, |probe| pack_lights(level, probe, sections.as_deref()))
      .map_err(XrfError::new_asset_error)?;
    let mut textures = self.textures.write().unwrap_or_else(PoisonError::into_inner);

    for projector in packed.projectors {
      textures.insert(projector.reference, projector.logical_path);
    }

    Ok(packed.lights)
  }
}
