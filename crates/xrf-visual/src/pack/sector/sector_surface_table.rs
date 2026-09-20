use xrf_level::{LevelShaderEntry, LevelShadersChunk};

use crate::data::sector::sector_surface::SectorSurface;

/// How a level's shader table dresses a surface, read by the id a drawable carries.
pub(crate) struct SectorSurfaceTable<'a> {
  entries: Option<&'a LevelShadersChunk>,
}

impl<'a> SectorSurfaceTable<'a> {
  /// The slot the deferred renderer reads hemisphere and sun occlusion out of.
  const HEMI_TEXTURE_SLOT: usize = 2;

  /// What that third texture has to start with before the engine calls the row lightmapped.
  const HEMI_TEXTURE_PREFIX: &'static str = "lmap";

  /// Reads a level's table, which a level without one answers nothing from.
  pub(crate) const fn of(entries: Option<&'a LevelShadersChunk>) -> Self {
    Self { entries }
  }

  /// How one entry dresses a surface, by the id a drawable carries.
  pub(crate) fn get(&self, shader_id: u16) -> SectorSurface {
    let Some(LevelShaderEntry::Reference(reference)) = self.entries.and_then(|it| it.entries.get(shader_id as usize))
    else {
      return SectorSurface {
        shader_id,
        ..SectorSurface::default()
      };
    };

    SectorSurface {
      hemi: Self::get_hemi(&reference.textures),
      shader_id,
      shader_name: Some(reference.shader.clone()),
      texture_name: reference.textures.first().cloned(),
    }
  }

  /// The occlusion texture a row names, or `None` for a row the engine's own test would not call lightmapped.
  fn get_hemi(textures: &[String]) -> Option<String> {
    textures
      .get(Self::HEMI_TEXTURE_SLOT)
      .filter(|name| name.starts_with(Self::HEMI_TEXTURE_PREFIX))
      .cloned()
  }
}
